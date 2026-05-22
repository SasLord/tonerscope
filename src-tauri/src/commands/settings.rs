// src-tauri/src/commands/settings.rs

use crate::db::{Database, models::AppSettings};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_settings(db: State<'_, Mutex<Database>>) -> Result<AppSettings, String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .get_all_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(db: State<'_, Mutex<Database>>, settings: AppSettings) -> Result<(), String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .save_all_settings(&settings)
        .map_err(|e| e.to_string())
}
