// src-tauri/src/lib.rs

mod commands;
mod db;
mod scanner;
mod scheduler;
mod snmp;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize SQLite database
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("tonerscope.db");

            let db = db::Database::new(&db_path).expect("Failed to init database");
            app.manage(std::sync::Mutex::new(db));

            // Start background scheduler
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::start(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::printer::get_printers,
            commands::printer::add_printer,
            commands::printer::remove_printer,
            commands::printer::poll_printer,
            commands::printer::get_snapshots,
            commands::printer::get_history_stats,   // <-- Фаза 3
            commands::scanner::scan_network,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TonerScope");
}
