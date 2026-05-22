// src-tauri/src/db/mod.rs

pub mod models;

use rusqlite::{Connection, Result, params};
use std::path::Path;
use models::{PrinterRecord, SnapshotRecord, AppSettings};

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
}
