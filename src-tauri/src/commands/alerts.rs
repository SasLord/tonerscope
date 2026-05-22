// src-tauri/src/commands/alerts.rs

use crate::db::{models::AlertRule, Database};
use std::sync::Mutex;
use tauri::State;

/// Возвращает все правила алертов.
#[tauri::command]
pub fn get_alert_rules(db: State<'_, Mutex<Database>>) -> Result<Vec<AlertRule>, String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .get_alert_rules()
        .map_err(|e| e.to_string())
}

/// Создаёт или обновляет правило алерта.
/// Фронтенд генерирует UUID v4 и передаёт полный объект.
#[tauri::command]
pub fn save_alert_rule(
    db: State<'_, Mutex<Database>>,
    rule: AlertRule,
) -> Result<(), String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .save_alert_rule(&rule)
        .map_err(|e| e.to_string())
}

/// Удаляет правило по id.
#[tauri::command]
pub fn delete_alert_rule(
    db: State<'_, Mutex<Database>>,
    id: String,
) -> Result<(), String> {
    db.lock()
        .map_err(|e| e.to_string())?
        .delete_alert_rule(&id)
        .map_err(|e| e.to_string())
}
