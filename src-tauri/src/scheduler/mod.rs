// src-tauri/src/scheduler/mod.rs

use crate::db::{models::AlertRule, Database};
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

    let (printers, settings, alert_rules) = {
        let d = match db.lock() {
            Ok(d) => d,
            Err(_) => return,
        };
        let p = d.get_printers().unwrap_or_default();
        let s = d.get_all_settings().unwrap_or_default();
        let r = d.get_alert_rules().unwrap_or_default();
        (p, s, r)
    };

    log::info!("Polling {} printers, {} alert rules", printers.len(), alert_rules.len());

    for printer in printers {
        let cfg = SnmpConfig {
            community: settings.snmp_community.clone(),
            timeout:   Duration::from_secs(settings.snmp_timeout),
            retries:   settings.snmp_retries,
        };
        let low_th  = settings.low_toner_threshold;
        let crit_th = settings.critical_toner_threshold;
        let ip      = printer.ip.clone();
        let printer_id = printer.id.clone();
        let printer_name = printer.name.clone();
        let app2    = app.clone();
        let rules   = alert_rules.clone();

        tokio::task::spawn_blocking(move || {
            let client   = SnmpClient::new(cfg);
            let snapshot = client.poll(&ip, low_th, crit_th);

            // Emit update to frontend
            let _ = app2.emit("printer-updated", &snapshot);

            // Check each supply against active alert rules
            for supply in &snapshot.supplies {
                // Emit legacy printer-alert event (критические — для Toast в layout.svelte)
                if supply.is_critical {
                    let _ = app2.emit("printer-alert", serde_json::json!({
                        "ip":      &ip,
                        "supply":  &supply.name,
                        "percent": supply.percent,
                    }));
                }

                // Desktop-уведомления по правилам из БД
                let matching_rules: Vec<&AlertRule> = rules
                    .iter()
                    .filter(|r| {
                        r.enabled
                            && r.notify_desktop
                            // совпадение по принтеру
                            && (r.printer_id == "all" || r.printer_id == printer_id)
                            // совпадение по типу расходника
                            && (r.supply_type == "any" || r.supply_type == supply.supply_type)
                            // порог сработал
                            && i32::from(supply.percent) <= r.threshold as i32
                    })
                    .collect();

                if !matching_rules.is_empty() {
                    send_desktop_notification(
                        &app2,
                        &printer_name,
                        &supply.name,
                        supply.percent.into(),
                    );
                }
            }
        });
    }
}

/// Отправляет нативное desktop-уведомление через tauri-plugin-notification.
fn send_desktop_notification(app: &AppHandle, printer_name: &str, supply_name: &str, percent: i32) {
    use tauri_plugin_notification::NotificationExt;

    let title = "TonerScope — Низкий уровень расходника".to_string();
    let body  = format!("{} · {} → {}%", printer_name, supply_name, percent);

    if let Err(e) = app
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
    {
        log::warn!("Failed to send desktop notification: {e}");
    }
}
