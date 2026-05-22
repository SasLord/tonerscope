// src/lib/index.ts

// ─── Components ───────────────────────────────────────────────────────────────
export * from './components/ui/index';
export * from './components/printer/index';
export * from './components/layout/index';

// ─── Stores ───────────────────────────────────────────────────────────────────
export { printers, printerStats, scanResults, isScanning, selectedPrinterId } from './stores/printers';
export { settings, theme, resolvedTheme }                                      from './stores/settings';
export { notifications }                                                        from './stores/notifications';

// ─── Types ────────────────────────────────────────────────────────────────────
export type {
  PrinterInfo,
  PrinterStatus,
  PrinterBrand,
  Supply,
  SupplyType,
  PrinterSnapshot,
  ScanResult,
  AlertRule,
  AppSettings,
  NavPage,
} from './types/printer';

// ─── Utils ────────────────────────────────────────────────────────────────────
export { formatPageCount, formatRelativeTime, statusLabel, brandLabel, supplyLabel } from './utils/formatters';
export { tonerColor, statusColor, supplyTypeColor, brandColor }                      from './utils/colors';
