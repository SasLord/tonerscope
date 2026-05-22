// src-tauri/src/commands/scanner.rs

use crate::db::Database;
use crate::scanner::{NetworkScanner, ScanResult};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn scan_network(
    app: AppHandle,
    db:  State<'_, Mutex<Database>>,
    subnet: String,
) -> Result<Vec<ScanResult>, String> {
    let community = {
        let d = db.lock().map_err(|e| e.to_string())?;
        d.get_all_settings()
            .map_err(|e| e.to_string())?
            .snmp_community
    };

    let app2 = app.clone();
    let scanner = NetworkScanner::new(&community);

    let results = scanner
        .scan_subnet(&subnet, move |done, total| {
            let progress = (done as f32 / total as f32 * 100.0) as u32;
            let _ = app2.emit("scan-progress", progress);
        })
        .await;

    Ok(results)
}
