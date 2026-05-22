// src-tauri/src/scheduler/mod.rs

use crate::db::Database;
use crate::snmp::{SnmpClient, SnmpConfig};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub async fn start(app: AppHandle) {
    log::info!("Scheduler started");

    loop {
        let interval_mins = {
            let db = app.state::<Mutex<Database>>();
            db.lock()
                .ok()
                .and_then(|d| d.get_all_settings().ok())
                .map(|s| s.poll_interval_minutes)
                .unwrap_or(5)
        };

        tokio::time::sleep(Duration::from_secs(interval_mins as u64 * 60)).await;

        poll_all(&app).await;
    }
}

async fn poll_all(app: &AppHandle) {
    let db = app.state::<Mutex<Database>>();

    let (printers, settings) = {
        let d = match db.lock() {
            Ok(d) => d,
            Err(_) => return,
        };
        let p = d.get_printers().unwrap_or_default();
        let s = d.get_all_settings().unwrap_or_default();
        (p, s)
    };

    log::info!("Polling {} printers", printers.len());

    for printer in printers {
        let cfg = SnmpConfig {
            community: settings.snmp_community.clone(),
            timeout:   Duration::from_secs(settings.snmp_timeout),
            retries:   settings.snmp_retries,
        };
        let low_th  = settings.low_toner_threshold;
        let crit_th = settings.critical_toner_threshold;
        let ip      = printer.ip.clone();
        let app2    = app.clone();

        tokio::task::spawn_blocking(move || {
            let client = SnmpClient::new(cfg);
            let snapshot = client.poll(&ip, low_th, crit_th);

            // Emit update to frontend
            let _ = app2.emit("printer-updated", &snapshot);

            // Send desktop notification if critical
            for supply in &snapshot.supplies {
                if supply.is_critical {
                    let _ = app2.emit("printer-alert", serde_json::json!({
                        "ip":      &ip,
                        "supply":  &supply.name,
                        "percent": supply.percent,
                    }));
                }
            }
        });
    }
}
