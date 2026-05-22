// src/lib/stores/printers.ts

import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';
import type { PrinterInfo, PrinterStatus, ScanResult, Supply, SupplyType, PrinterBrand } from '$lib/types/printer';
import { api, type PrinterRecord, type PrinterSnapshotRecord, type SupplyRecord } from '$lib/api/tauri';

// ─── Helpers ─────────────────────────────────────────────────────────────────

function mapSupply(s: SupplyRecord): Supply {
  return {
    type:       s.type      as SupplyType,
    name:       s.name,
    level:      s.level,
    maxLevel:   s.maxLevel,
    percent:    s.percent,
    isLow:      s.isLow,
    isCritical: s.isCritical,
  };
}

function recordToPrinterInfo(r: PrinterRecord): PrinterInfo {
  return {
    id:            r.id,
    ip:            r.ip,
    name:          r.name,
    brand:         r.brand as PrinterBrand,
    model:         r.model,
    location:      r.location,
    group:         r.group,
    status:        'unknown',
    supplies:      [],
    lastSeen:      new Date().toISOString(),
    addedManually: r.addedManually,
  };
}

export function mergeSnapshot(printer: PrinterInfo, snap: PrinterSnapshotRecord): PrinterInfo {
  return {
    ...printer,
    status:    snap.status    as PrinterStatus,
    supplies:  snap.supplies.map(mapSupply),
    pageCount: snap.pageCount,
    lastSeen:  snap.timestamp,
  };
}

// ─── Store ───────────────────────────────────────────────────────────────────

function createPrinterStore() {
  const { subscribe, set, update } = writable<PrinterInfo[]>([]);

  return {
    subscribe, set, update,

    upsert: (printer: PrinterInfo) =>
      update(list => {
        const idx = list.findIndex(p => p.id === printer.id);
        if (idx >= 0) { const next = [...list]; next[idx] = printer; return next; }
        return [...list, printer];
      }),

    remove: (id: string) =>
      update(list => list.filter(p => p.id !== id)),

    updateStatus: (id: string, status: PrinterStatus) =>
      update(list => list.map(p => p.id === id ? { ...p, status } : p)),

    mergeSnapshotByIp: (snap: PrinterSnapshotRecord) =>
      update(list => list.map(p =>
        p.ip === snap.printerId ? mergeSnapshot(p, snap) : p
      )),
  };
}

export const printers = createPrinterStore();

// ─── Инициализация ───────────────────────────────────────────────────────────

let unlistenUpdated: (() => void) | null = null;

export async function initPrinters(): Promise<void> {
  if (!browser) return;

  try {
    const records = await api.getPrinters();
    printers.set(records.map(recordToPrinterInfo));
  } catch (err) {
    console.error('[printers] getPrinters failed:', err);
  }

  unlistenUpdated = await api.onPrinterUpdated(snap => {
    printers.mergeSnapshotByIp(snap);
  });
}

export function destroyPrinters(): void {
  unlistenUpdated?.();
  unlistenUpdated = null;
}

// ─── Derived ─────────────────────────────────────────────────────────────────

export const printerStats = derived(printers, ($printers) => ({
  total:    $printers.length,
  online:   $printers.filter(p => p.status === 'online' || p.status === 'printing').length,
  offline:  $printers.filter(p => p.status === 'offline').length,
  errors:   $printers.filter(p => p.status === 'error').length,
  warnings: $printers.filter(p => p.status === 'warning').length,
  lowToner: $printers.filter(p => p.supplies.some(s => s.isLow)).length,
  critical: $printers.filter(p => p.supplies.some(s => s.isCritical)).length,
}));

export const scanResults       = writable<ScanResult[]>([]);
export const isScanning        = writable<boolean>(false);
export const selectedPrinterId = writable<string | null>(null);
