// src-tauri/src/db/mod.rs

pub mod models;

use rusqlite::{Connection, Result, params};
use std::path::Path;
use models::{
    PrinterRecord, SnapshotRecord, AppSettings,
    HistoryStatsRecord, SupplyStatRecord, AlertRule,
};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS printers (
                id             TEXT PRIMARY KEY,
                ip             TEXT NOT NULL UNIQUE,
                name           TEXT NOT NULL,
                brand          TEXT NOT NULL DEFAULT 'other',
                model          TEXT NOT NULL DEFAULT '',
                location       TEXT,
                grp            TEXT,
                added_manually INTEGER NOT NULL DEFAULT 0,
                created_at     TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS snapshots (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                printer_id    TEXT NOT NULL REFERENCES printers(id) ON DELETE CASCADE,
                timestamp     TEXT NOT NULL DEFAULT (datetime('now')),
                status        TEXT NOT NULL,
                page_count    INTEGER,
                supplies_json TEXT NOT NULL DEFAULT '[]'
            );

            CREATE INDEX IF NOT EXISTS idx_snapshots_printer_ts
                ON snapshots(printer_id, timestamp DESC);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS alert_rules (
                id             TEXT PRIMARY KEY,
                printer_id     TEXT NOT NULL DEFAULT 'all',
                supply_type    TEXT NOT NULL DEFAULT 'any',
                threshold      INTEGER NOT NULL DEFAULT 20,
                enabled        INTEGER NOT NULL DEFAULT 1,
                notify_desktop INTEGER NOT NULL DEFAULT 1
            );
        ")?;
        Ok(())
    }

    // ── Printers ──────────────────────────────────────────────────────────────

    pub fn upsert_printer(&self, p: &PrinterRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO printers (id, ip, name, brand, model, location, grp, added_manually)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               ip = excluded.ip,
               name = excluded.name,
               brand = excluded.brand,
               model = excluded.model,
               location = excluded.location,
               grp = excluded.grp,
               updated_at = datetime('now')",
            params![
                p.id, p.ip, p.name, p.brand, p.model,
                p.location, p.group, p.added_manually as i32
            ],
        )?;
        Ok(())
    }

    pub fn get_printers(&self) -> Result<Vec<PrinterRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ip, name, brand, model, location, grp, added_manually
             FROM printers ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PrinterRecord {
                id:             row.get(0)?,
                ip:             row.get(1)?,
                name:           row.get(2)?,
                brand:          row.get(3)?,
                model:          row.get(4)?,
                location:       row.get(5)?,
                group:          row.get(6)?,
                added_manually: row.get::<_, i32>(7)? != 0,
            })
        })?;
        rows.collect()
    }

    pub fn remove_printer(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM printers WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Snapshots ─────────────────────────────────────────────────────────────

    pub fn insert_snapshot(&self, s: &SnapshotRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO snapshots (printer_id, status, page_count, supplies_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![s.printer_id, s.status, s.page_count, s.supplies_json],
        )?;
        Ok(())
    }

    /// Возвращает последние `limit` снапшотов для принтера,
    /// отсортированных по времени (новые первые).
    pub fn get_snapshots(&self, printer_id: &str, limit: i64) -> Result<Vec<SnapshotRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, printer_id, timestamp, status, page_count, supplies_json
             FROM snapshots
             WHERE printer_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![printer_id, limit], |row| {
            Ok(SnapshotRecord {
                id:            Some(row.get(0)?),
                printer_id:    row.get(1)?,
                timestamp:     row.get(2)?,
                status:        row.get(3)?,
                page_count:    row.get(4)?,
                supplies_json: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    // ── History Stats (Фаза 3) ────────────────────────────────────────────────

    /// Агрегирует историю по расходникам за `period_days` дней.
    /// period_days = 0 означает «всё время».
    /// Прогноз вычисляется методом наименьших квадратов по последним 30 точкам.
    pub fn get_history_stats(
        &self,
        printer_id: &str,
        period_days: i64,
    ) -> Result<HistoryStatsRecord> {
        let since_clause = if period_days > 0 {
            format!(
                "AND timestamp >= datetime('now', '-{} days')",
                period_days
            )
        } else {
            String::new()
        };

        let snapshot_count: i64 = self.conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM snapshots WHERE printer_id = ?1 {since_clause}"
            ),
            params![printer_id],
            |r| r.get(0),
        )?;

        let mut stmt = self.conn.prepare(&format!(
            "SELECT timestamp, supplies_json
             FROM snapshots
             WHERE printer_id = ?1 {since_clause}
             ORDER BY timestamp ASC"
        ))?;

        let mut supply_series: std::collections::HashMap<
            String,
            (String, Vec<(f64, i64)>),
        > = std::collections::HashMap::new();

        let epoch = chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
            .expect("static parse")
            .timestamp() as f64;

        let rows = stmt.query_map(params![printer_id], |row| {
            let ts: String   = row.get(0)?;
            let json: String = row.get(1)?;
            Ok((ts, json))
        })?;

        for row in rows {
            let (ts, json) = row?;

            let unix_days = chrono::DateTime::parse_from_rfc3339(&ts)
                .map(|dt| (dt.timestamp() as f64 - epoch) / 86400.0)
                .unwrap_or(0.0);

            let supplies_raw: Vec<serde_json::Value> =
                serde_json::from_str(&json).unwrap_or_default();

            for s in &supplies_raw {
                let stype = s["type"].as_str().unwrap_or("other").to_string();
                let sname = s["name"].as_str().unwrap_or(&stype).to_string();
                let pct   = s["percent"].as_i64().unwrap_or(0);

                supply_series
                    .entry(stype)
                    .or_insert_with(|| (sname, Vec::new()))
                    .1
                    .push((unix_days, pct));
            }
        }

        let mut supplies: Vec<SupplyStatRecord> = supply_series
            .into_iter()
            .map(|(stype, (sname, pts))| {
                let pcts: Vec<i64> = pts.iter().map(|(_, p)| *p).collect();

                let min_pct   = *pcts.iter().min().unwrap_or(&0);
                let max_pct   = *pcts.iter().max().unwrap_or(&0);
                let avg_pct   = if pcts.is_empty() { 0 } else {
                    pcts.iter().sum::<i64>() / pcts.len() as i64
                };
                let first_pct = pcts.first().copied().unwrap_or(0);
                let last_pct  = pcts.last().copied().unwrap_or(0);
                let snapshot_count = pcts.len() as i64;

                let forecast_days = if pts.len() >= 3 {
                    compute_forecast_days(&pts)
                } else {
                    None
                };

                SupplyStatRecord {
                    supply_type: stype,
                    supply_name: sname,
                    min_pct,
                    max_pct,
                    avg_pct,
                    first_pct,
                    last_pct,
                    snapshot_count,
                    forecast_days,
                }
            })
            .collect();

        let order = |t: &str| match t {
            "toner_black"   => 0u8,
            "toner_cyan"    => 1,
            "toner_magenta" => 2,
            "toner_yellow"  => 3,
            "drum"          => 4,
            "fuser"         => 5,
            "waste"         => 6,
            _               => 7,
        };
        supplies.sort_by_key(|s| order(&s.supply_type));

        Ok(HistoryStatsRecord {
            printer_id:     printer_id.to_string(),
            period_days,
            snapshot_count,
            supplies,
        })
    }

    // ── Settings ──────────────────────────────────────────────────────────────

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .map(Some)
            .or_else(|e| {
                if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) }
            })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<AppSettings> {
        let json = self.get_setting("app_settings")?.unwrap_or_default();
        if json.is_empty() {
            return Ok(AppSettings::default());
        }
        serde_json::from_str(&json)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
    }

    pub fn save_all_settings(&self, s: &AppSettings) -> Result<()> {
        let json = serde_json::to_string(s)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        self.set_setting("app_settings", &json)
    }

    // ── Alert Rules (Фаза 4) ──────────────────────────────────────────────────

    /// Возвращает все правила алертов, отсортированные по threshold DESC.
    pub fn get_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, printer_id, supply_type, threshold, enabled, notify_desktop
             FROM alert_rules
             ORDER BY threshold DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(AlertRule {
                id:             row.get(0)?,
                printer_id:     row.get(1)?,
                supply_type:    row.get(2)?,
                threshold:      row.get(3)?,
                enabled:        row.get::<_, i32>(4)? != 0,
                notify_desktop: row.get::<_, i32>(5)? != 0,
            })
        })?;
        rows.collect()
    }

    /// Вставляет или обновляет правило алерта (upsert по id).
    pub fn save_alert_rule(&self, r: &AlertRule) -> Result<()> {
        self.conn.execute(
            "INSERT INTO alert_rules (id, printer_id, supply_type, threshold, enabled, notify_desktop)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               printer_id     = excluded.printer_id,
               supply_type    = excluded.supply_type,
               threshold      = excluded.threshold,
               enabled        = excluded.enabled,
               notify_desktop = excluded.notify_desktop",
            params![
                r.id,
                r.printer_id,
                r.supply_type,
                r.threshold,
                r.enabled as i32,
                r.notify_desktop as i32,
            ],
        )?;
        Ok(())
    }

    /// Удаляет правило по id.
    pub fn delete_alert_rule(&self, id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM alert_rules WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
}

// ─── Вспомогательная функция: МНК-прогноз ────────────────────────────────────

/// Принимает вектор (unix_days, pct). Использует последние 30 точек.
/// Возвращает количество дней от последней точки до достижения 0%,
/// или None если тренд восходящий / плоский.
fn compute_forecast_days(pts: &[(f64, i64)]) -> Option<i64> {
    let slice = if pts.len() > 30 { &pts[pts.len() - 30..] } else { pts };
    let n = slice.len() as f64;

    let xs: Vec<f64> = slice.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f64> = slice.iter().map(|(_, y)| *y as f64).collect();

    let sum_x  = xs.iter().sum::<f64>();
    let sum_y  = ys.iter().sum::<f64>();
    let sum_xy = xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum::<f64>();
    let sum_x2 = xs.iter().map(|x| x * x).sum::<f64>();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-9 {
        return None;
    }

    let slope     = (n * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n;

    if slope >= -0.01 {
        return None;
    }

    let last_x   = xs[xs.len() - 1];
    let days_raw = (0.0 - intercept) / slope - last_x;

    if days_raw < 0.0 {
        Some(0)
    } else {
        Some(days_raw.round() as i64)
    }
}
