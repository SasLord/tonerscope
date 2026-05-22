// src/lib/utils/colors.ts

import type { PrinterBrand, PrinterStatus, SupplyType } from '$lib/types/printer';

export function tonerColor(percent: number): string {
  if (percent <= 10) return 'var(--gauge-crit)';
  if (percent <= 20) return 'var(--gauge-low)';
  return 'var(--gauge-ok)';
}

export function statusColor(status: PrinterStatus): string {
  const map: Record<PrinterStatus, string> = {
    online:   'var(--status-online)',
    offline:  'var(--status-offline)',
    printing: 'var(--status-printing)',
    error:    'var(--status-error)',
    warning:  'var(--status-warning)',
    unknown:  'var(--status-unknown)',
  };
  return map[status] ?? 'var(--status-unknown)';
}

export function supplyTypeColor(type: SupplyType): string {
  const map: Record<SupplyType, string> = {
    toner_black:   'var(--text-primary)',
    toner_cyan:    '#06b6d4',
    toner_magenta: '#ec4899',
    toner_yellow:  '#eab308',
    drum:          '#8b5cf6',
    fuser:         '#f97316',
    waste:         'var(--text-tertiary)',
    other:         'var(--text-secondary)',
  };
  return map[type] ?? 'var(--text-secondary)';
}

export function brandColor(brand: PrinterBrand): string {
  const map: Record<PrinterBrand, string> = {
    pantum:  '#0066cc',
    kyocera: '#cc0000',
    hp:      '#0096d6',
    canon:   '#cc0000',
    other:   'var(--text-tertiary)',
  };
  return map[brand] ?? 'var(--text-tertiary)';
}
