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

// AppSettings без theme (Rust-структура не содержит это поле)
export type AppSettingsRecord = Omit<AppSettings, 'theme'>;

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
  // Rust принимает snake_case имена аргументов команды (не serde-поля)
  getSnapshots: (printerId: string, limit = 90): Promise<SnapshotRecord[]> =>
    inv('get_snapshots', { printer_id: printerId, limit }),

  // ── Сканирование ──────────────────────────────────────────────────────────
  scanNetwork: (subnet: string): Promise<ScanResult[]> =>
    inv('scan_network', { subnet }),

  // ── Настройки ─────────────────────────────────────────────────────────────
  getSettings: (): Promise<AppSettingsRecord> =>
    inv('get_settings'),

  saveSettings: (settings: AppSettingsRecord): Promise<void> =>
    inv('save_settings', { settings }),

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
