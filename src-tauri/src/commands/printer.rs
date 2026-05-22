// src-tauri/src/commands/printer.rs

use crate::db::{Database, models::PrinterRecord, models::SnapshotRecord, models::HistoryStatsRecord};
use crate::snmp::{SnmpClient, SnmpConfig, PrinterSnapshot};
use chrono::Utc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use uuid::Uuid;

type DbState<'a> = State<'a, Mutex<Database>>;

#[tauri::command]
pub fn get_printers(db: DbState) -> Result<Vec<PrinterRecord>, String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .get_printers()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_printer(
    db:       DbState,
    ip:       String,
    name:     String,
    brand:    String,
    model:    String,
    location: Option<String>,
    group:    Option<String>,
) -> Result<PrinterRecord, String> {
    let record = PrinterRecord {
        id:             Uuid::new_v4().to_string(),
        ip,
        name,
        brand,
        model,
        location,
        group,
        added_manually: true,
    };
    db.lock()
        .map_err(|e| e.to_string())?
        .upsert_printer(&record)
        .map_err(|e| e.to_string())?;
    Ok(record)
}

#[tauri::command]
pub fn remove_printer(db: DbState, id: String) -> Result<(), String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .remove_printer(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_printer(
    db: State<'_, Mutex<Database>>,
    ip: String,
) -> Result<PrinterSnapshot, String> {
    let (community, low_th, crit_th) = {
        let d = db.lock().map_err(|e| e.to_string())?;
        let s = d.get_all_settings().map_err(|e| e.to_string())?;
        (s.snmp_community, s.low_toner_threshold, s.critical_toner_threshold)
    };

    let cfg = SnmpConfig {
        community,
        timeout: Duration::from_secs(3),
        retries: 2,
    };

    let ip_clone = ip.clone();
    let snapshot = tokio::task::spawn_blocking(move || {
        let client = SnmpClient::new(cfg);
        client.poll(&ip_clone, low_th, crit_th)
    })
    .await
    .map_err(|e| e.to_string())?;

    // Сохраняем снапшот в SQLite
    {
        let supplies_json = serde_json::to_string(&snapshot.supplies)
            .unwrap_or_else(|_| "[]".into());
        let d = db.lock().map_err(|e| e.to_string())?;
        let _ = d.insert_snapshot(&SnapshotRecord {
            id:            None,
            printer_id:    ip.clone(),
            timestamp:     Utc::now().to_rfc3339(),
            status:        snapshot.status.clone(),
            page_count:    snapshot.page_count,
            supplies_json,
        });
    }

    Ok(snapshot)
}

/// Возвращает последние `limit` снапшотов для принтера по его id.
/// Используется страницей истории тонера.
#[tauri::command]
pub fn get_snapshots(
    db:         DbState,
    printer_id: String,
    limit:      Option<i64>,
) -> Result<Vec<SnapshotRecord>, String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .get_snapshots(&printer_id, limit.unwrap_or(365))
        .map_err(|e| e.to_string())
}

/// Возвращает агрегированную статистику истории тонера по принтеру.
/// period_days: 7 | 30 | 90 | 0 (0 = всё время).
/// Включает МНК-прогноз дней до 0% для каждого расходника.
#[tauri::command]
pub fn get_history_stats(
    db:          DbState,
    printer_id:  String,
    period_days: Option<i64>,
) -> Result<HistoryStatsRecord, String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .get_history_stats(&printer_id, period_days.unwrap_or(30))
        .map_err(|e| e.to_string())
}
