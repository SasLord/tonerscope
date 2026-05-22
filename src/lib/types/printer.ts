// src/lib/types/printer.ts

export type PrinterStatus = 'online' | 'offline' | 'printing' | 'error' | 'warning' | 'unknown';

export type SupplyType = 'toner_black' | 'toner_cyan' | 'toner_magenta' | 'toner_yellow' | 'drum' | 'fuser' | 'waste' | 'other';

export type PrinterBrand = 'pantum' | 'kyocera' | 'hp' | 'canon' | 'other';

export interface Supply {
  type: SupplyType;
  name: string;
  level: number;       // текущий уровень (raw)
  maxLevel: number;    // максимум
  percent: number;     // 0–100
  isLow: boolean;      // < порог
  isCritical: boolean; // < критический порог
}

export interface PrinterInfo {
  id: string;
  ip: string;
  name: string;
  brand: PrinterBrand;
  model: string;
  location?: string;
  group?: string;
  status: PrinterStatus;
  supplies: Supply[];
  pageCount?: number;
  lastSeen: string;    // ISO date string
  addedManually: boolean;
}

export interface PrinterSnapshot {
  printerId: string;
  timestamp: string;
  supplies: Supply[];
  pageCount?: number;
  status: PrinterStatus;
}

export interface ScanResult {
  ip: string;
  isReachable: boolean;
  isSNMPOpen: boolean;
  sysDescr?: string;
  brand?: PrinterBrand;
  model?: string;
}

export interface AlertRule {
  id: string;
  printerId: string | 'all';
  supplyType: SupplyType | 'any';
  threshold: number;   // процент
  enabled: boolean;
  notifyDesktop: boolean;
}

export interface AppSettings {
  pollIntervalMinutes: number;
  lowTonerThreshold: number;
  criticalTonerThreshold: number;
  snmpCommunity: string;
  snmpTimeout: number;
  snmpRetries: number;
  theme: 'dark' | 'light' | 'system';
  subnets: string[];
}

export type NavPage = 'dashboard' | 'printers' | 'scan' | 'history' | 'alerts' | 'settings';
