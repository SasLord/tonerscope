// src/lib/api/index.ts

export { api, settingsToRecord, recordToSettings } from './tauri';
export type {
  PrinterRecord,
  SupplyRecord,
  PrinterSnapshotRecord,
  SnapshotRecord,
  AppSettingsRecord,
  ScanProgressPayload,
  PrinterUpdatedPayload,
  PrinterAlertPayload,
} from './tauri';
