// src/lib/api/tauri.ts
// Типизированная обёртка над Tauri IPC invoke/listen.
//
// ВАЖНО: @tauri-apps/api нельзя импортировать статически на верхнем уровне —
// SvelteKit с prerender=true пытается выполнить модуль в Node.js (SSR/prerender),
// где window.__TAURI__ отсутствует и происходит краш.
// Поэтому используем динамический import() только внутри функций,
// которые вызываются исключительно в браузере (onMount / обработчики событий).

import { browser } from '$app/environment';
import type { AppSettings, ScanResult } from '$lib/types/printer';

// ─── Типы, которые возвращает Rust-бэкенд (camelCase через serde) ─────────────

export interface PrinterRecord {
  id:            string;
  ip:            string;
  name:          string;
  brand:         string;
  model:         string;
  location?:     string;
  group?:        string;
  addedManually: boolean;
}

export interface SupplyRecord {
  type:       string;
  name:       string;
  level:      number;
  maxLevel:   number;
  percent:    number;
  isLow:      boolean;
  isCritical: boolean;
}

export interface PrinterSnapshotRecord {
  printerId:  string;
  timestamp:  string;
  status:     string;
  pageCount?: number;
  supplies:   SupplyRecord[];
}

export interface SnapshotRecord {
  id?:          number;
  printerId:    string;
  timestamp:    string;
  status:       string;
  pageCount?:   number;
  suppliesJson: string;
}

// ─── Фаза 3: статистика истории по расходнику ─────────────────────────────────

export interface SupplyStatRecord {
  supplyType:    string;
  supplyName:    string;
  minPct:        number;
  maxPct:        number;
  avgPct:        number;
  firstPct:      number;
  lastPct:       number;
  snapshotCount: number;
  /** Прогноз дней до 0% (null если нет тренда или данных недостаточно) */
  forecastDays:  number | null;
}

export interface HistoryStatsRecord {
  printerId:     string;
  periodDays:    number;
  snapshotCount: number;
  supplies:      SupplyStatRecord[];
}

// AppSettings без theme (Rust-структура не содержит это поле)
export type AppSettingsRecord = Omit<AppSettings, 'theme'>;

// ─── Фаза 4: правила алертов ──────────────────────────────────────────────────

export interface AlertRuleRecord {
  id:            string;
  /** UUID принтера или "all" */
  printerId:     string;
  /** Тип расходника (toner_black, drum, …) или "any" */
  supplyType:    string;
  /** Порог срабатывания (%) */
  threshold:     number;
  enabled:       boolean;
  notifyDesktop: boolean;
}

// ─── Фаза 6.1: Print Spooler ──────────────────────────────────────────────────

export interface SpoolerRestartResult {
  success: boolean;
  message: string;
  /** "running" | "stopped" | "start_pending" | "stop_pending" | "unknown" | "unavailable" */
  status:  string;
}

// ─── Payloads событий ─────────────────────────────────────────────────────────

export interface ScanProgressPayload {
  percent: number;
  current: string;
  found:   number;
}

export type PrinterUpdatedPayload = PrinterSnapshotRecord;

export interface PrinterAlertPayload {
  ip:      string;
  supply:  string;
  percent: number;
}

// ─── UnlistenFn тип без импорта модуля ────────────────────────────────────────
export type UnlistenFn = () => void;

// ─── Внутренний хелпер: безопасный invoke ────────────────────────────────────

async function inv<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!browser) throw new Error('invoke called outside browser');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

// ─── API ─────────────────────────────────────────────────────────────────────

export const api = {
  // ── Принтеры ──────────────────────────────────────────────────────────────
  getPrinters: (): Promise<PrinterRecord[]> =>
    inv('get_printers'),

  addPrinter: (params: {
    ip:        string;
    name:      string;
    brand:     string;
    model:     string;
    location?: string;
    group?:    string;
  }): Promise<PrinterRecord> =>
    inv('add_printer', params),

  removePrinter: (id: string): Promise<void> =>
    inv('remove_printer', { id }),

  pollPrinter: (ip: string): Promise<PrinterSnapshotRecord> =>
    inv('poll_printer', { ip }),

  // ── Снапшоты ──────────────────────────────────────────────────────────────
  getSnapshots: (printerId: string, limit = 365): Promise<SnapshotRecord[]> =>
    inv('get_snapshots', { printer_id: printerId, limit }),

  // ── История: агрегированная статистика (Фаза 3) ───────────────────────────
  getHistoryStats: (printerId: string, periodDays = 30): Promise<HistoryStatsRecord> =>
    inv('get_history_stats', { printer_id: printerId, period_days: periodDays }),

  // ── Сканирование ──────────────────────────────────────────────────────────
  scanNetwork: (subnet: string): Promise<ScanResult[]> =>
    inv('scan_network', { subnet }),

  // ── Настройки ─────────────────────────────────────────────────────────────
  getSettings: (): Promise<AppSettingsRecord> =>
    inv('get_settings'),

  saveSettings: (settings: AppSettingsRecord): Promise<void> =>
    inv('save_settings', { settings }),

  // ── Правила алертов (Фаза 4) ──────────────────────────────────────────────

  getAlertRules: (): Promise<AlertRuleRecord[]> =>
    inv('get_alert_rules'),

  /** Создаёт или обновляет правило. Фронтенд генерирует id через crypto.randomUUID(). */
  saveAlertRule: (rule: AlertRuleRecord): Promise<void> =>
    inv('save_alert_rule', { rule }),

  deleteAlertRule: (id: string): Promise<void> =>
    inv('delete_alert_rule', { id }),

  // ── Print Spooler (Фаза 6.1, Windows only) ───────────────────────────────

  /** Перезапускает службу Print Spooler. На не-Windows системах выбрасывает ошибку. */
  restartSpooler: (): Promise<SpoolerRestartResult> =>
    inv('restart_spooler'),

  /** Возвращает текущий статус службы Spooler: "running" / "stopped" / "unavailable" / … */
  getSpoolerStatus: (): Promise<string> =>
    inv('get_spooler_status'),

  // ── События Backend → Frontend ────────────────────────────────────────────

  onPrinterUpdated: async (
    cb: (payload: PrinterUpdatedPayload) => void
  ): Promise<UnlistenFn> => {
    if (!browser) return () => {};
    const { listen } = await import('@tauri-apps/api/event');
    return listen<PrinterUpdatedPayload>('printer-updated', e => cb(e.payload));
  },

  onPrinterAlert: async (
    cb: (payload: PrinterAlertPayload) => void
  ): Promise<UnlistenFn> => {
    if (!browser) return () => {};
    const { listen } = await import('@tauri-apps/api/event');
    return listen<PrinterAlertPayload>('printer-alert', e => cb(e.payload));
  },

  onScanProgress: async (
    cb: (payload: ScanProgressPayload) => void
  ): Promise<UnlistenFn> => {
    if (!browser) return () => {};
    const { listen } = await import('@tauri-apps/api/event');
    return listen<ScanProgressPayload>('scan-progress', e => cb(e.payload));
  },
};
