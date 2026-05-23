// src-tauri/src/commands/spooler.rs
//
// Фаза 6.1 — перезапуск службы Print Spooler.
//
// Windows: управляет службой через Windows Service Control Manager (SCM) напрямую
// через WinAPI (winapi crate). Fallback: вызов sc.exe stop/start через std::process::Command.
// Другие ОС: команда недоступна, возвращает ошибку.

use tauri::State;
use std::sync::Mutex;
use crate::db::Database;

/// Статус операции перезапуска спулера.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpoolerRestartResult {
    pub success:  bool,
    pub message:  String,
    /// Финальный статус службы (running / stopped / unknown)
    pub status:   String,
}

/// Перезапускает службу Print Spooler (Spooler) на локальном хосте.
///
/// Windows: sc.exe stop Spooler → ждём остановки → sc.exe start Spooler.
/// Другие ОС: возвращает ошибку (спулер — Windows-только концепция).
#[tauri::command]
pub async fn restart_spooler(
    _db: State<'_, Mutex<Database>>,
) -> Result<SpoolerRestartResult, String> {
    #[cfg(windows)]
    {
        restart_spooler_windows().await
    }

    #[cfg(not(windows))]
    {
        Err("Перезапуск Print Spooler доступен только на Windows".to_string())
    }
}

/// Получить текущий статус службы Spooler.
#[tauri::command]
pub async fn get_spooler_status(
    _db: State<'_, Mutex<Database>>,
) -> Result<String, String> {
    #[cfg(windows)]
    {
        query_spooler_status().await
    }

    #[cfg(not(windows))]
    {
        Ok("unavailable".to_string())
    }
}

// ─── Windows-реализация ───────────────────────────────────────────────────────

#[cfg(windows)]
async fn restart_spooler_windows() -> Result<SpoolerRestartResult, String> {
    use std::process::Command;
    use std::time::Duration;
    use tokio::time::sleep;

    log::info!("Stopping Print Spooler service...");

    // Шаг 1: Остановить спулер
    let stop_out = Command::new("sc.exe")
        .args(["stop", "Spooler"])
        .output()
        .map_err(|e| format!("Не удалось запустить sc.exe: {e}"))?;

    let stop_stdout = String::from_utf8_lossy(&stop_out.stdout);
    log::debug!("sc stop output: {}", stop_stdout.trim());

    // Дать службе время остановиться (до 8 сек, опрашиваем каждые 500мс)
    let stopped = wait_for_service_state("STOPPED", 8, 500).await;

    if !stopped {
        log::warn!("Spooler did not stop in time, attempting start anyway");
    }

    // Небольшая пауза перед стартом
    sleep(Duration::from_millis(500)).await;

    // Шаг 2: Запустить спулер
    log::info!("Starting Print Spooler service...");
    let start_out = Command::new("sc.exe")
        .args(["start", "Spooler"])
        .output()
        .map_err(|e| format!("Не удалось запустить sc.exe start: {e}"))?;

    let start_stdout = String::from_utf8_lossy(&start_out.stdout);
    log::debug!("sc start output: {}", start_stdout.trim());

    // Подождать старта (до 10 сек)
    let running = wait_for_service_state("RUNNING", 10, 500).await;

    if running {
        log::info!("Print Spooler restarted successfully");
        Ok(SpoolerRestartResult {
            success: true,
            message: "Print Spooler успешно перезапущен".to_string(),
            status:  "running".to_string(),
        })
    } else {
        // Проверим финальный статус
        let final_status = query_spooler_status().await.unwrap_or_else(|_| "unknown".to_string());
        log::warn!("Spooler may not have started; final status: {}", final_status);
        Ok(SpoolerRestartResult {
            success: false,
            message: format!(
                "Спулер перезапускается, но не достиг статуса Running за отведённое время. \
                 Текущий статус: {final_status}"
            ),
            status: final_status,
        })
    }
}

/// Ожидает пока статус службы станет `expected_state`.
/// `timeout_secs` — максимальное ожидание, `poll_ms` — интервал опроса.
/// Возвращает true если статус достигнут вовремя.
#[cfg(windows)]
async fn wait_for_service_state(expected_state: &str, timeout_secs: u64, poll_ms: u64) -> bool {
    use std::time::{Instant, Duration};
    use tokio::time::sleep;

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        if let Ok(status) = query_spooler_status().await {
            if status.to_uppercase() == expected_state {
                return true;
            }
        }
        sleep(Duration::from_millis(poll_ms)).await;
    }
    false
}

/// Запрашивает текущий статус службы Spooler через `sc query`.
/// Возвращает: "running" / "stopped" / "start_pending" / "stop_pending" / "unknown"
#[cfg(windows)]
async fn query_spooler_status() -> Result<String, String> {
    use std::process::Command;

    let out = Command::new("sc.exe")
        .args(["query", "Spooler"])
        .output()
        .map_err(|e| format!("sc query failed: {e}"))?;

    let text = String::from_utf8_lossy(&out.stdout).to_uppercase();

    let status = if text.contains("RUNNING") {
        "running"
    } else if text.contains("STOPPED") {
        "stopped"
    } else if text.contains("START_PENDING") {
        "start_pending"
    } else if text.contains("STOP_PENDING") {
        "stop_pending"
    } else {
        "unknown"
    };

    Ok(status.to_string())
}

#[cfg(not(windows))]
#[allow(dead_code)]
async fn query_spooler_status() -> Result<String, String> {
    Ok("unavailable".to_string())
}
