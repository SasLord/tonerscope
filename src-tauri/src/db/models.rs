// src-tauri/src/db/models.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterRecord {
    pub id:             String,
    pub ip:             String,
    pub name:           String,
    pub brand:          String,
    pub model:          String,
    pub location:       Option<String>,
    pub group:          Option<String>,
    pub added_manually: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotRecord {
    /// Автоинкремент SQLite. None при вставке, Some при чтении.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id:            Option<i64>,
    pub printer_id:    String,
    pub timestamp:     String,
    pub status:        String,
    pub page_count:    Option<i64>,
    pub supplies_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub poll_interval_minutes:    u32,
    pub low_toner_threshold:      u8,
    pub critical_toner_threshold: u8,
    pub snmp_community:           String,
    pub snmp_timeout:             u64,
    pub snmp_retries:             u32,
    pub subnets:                  Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            poll_interval_minutes:    5,
            low_toner_threshold:      20,
            critical_toner_threshold: 10,
            snmp_community:           "public".into(),
            snmp_timeout:             3,
            snmp_retries:             2,
            subnets:                  vec!["192.168.1.0/24".into()],
        }
    }
}

// ─── Фаза 3: агрегированная статистика истории тонера ──────────────────────────

/// Статистика по одному расходнику за период.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyStatRecord {
    pub supply_type:    String,
    pub supply_name:    String,
    pub min_pct:        i64,
    pub max_pct:        i64,
    pub avg_pct:        i64,
    pub first_pct:      i64,
    pub last_pct:       i64,
    pub snapshot_count: i64,
    /// Линейный прогноз: дней до достижения 0%.
    /// None — если данных < 3 или тренд восходящий/плоский.
    pub forecast_days:  Option<i64>,
}

/// Итоговый ответ команды get_history_stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStatsRecord {
    pub printer_id:     String,
    pub period_days:    i64,
    pub snapshot_count: i64,
    pub supplies:       Vec<SupplyStatRecord>,
}
