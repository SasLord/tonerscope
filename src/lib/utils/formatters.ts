// src/lib/utils/formatters.ts

import type { PrinterBrand, PrinterStatus, SupplyType } from '$lib/types/printer';

export function formatPageCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000)     return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

export function formatRelativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const secs  = Math.floor(diff / 1000);
  const mins  = Math.floor(secs / 60);
  const hours = Math.floor(mins / 60);
  const days  = Math.floor(hours / 24);

  if (secs < 60)   return 'только что';
  if (mins < 60)   return `${mins} мин. назад`;
  if (hours < 24)  return `${hours} ч. назад`;
  return `${days} дн. назад`;
}

export function statusLabel(status: PrinterStatus): string {
  const map: Record<PrinterStatus, string> = {
    online:   'В сети',
    offline:  'Не в сети',
    printing: 'Печатает',
    error:    'Ошибка',
    warning:  'Предупреждение',
    unknown:  'Неизвестно',
  };
  return map[status] ?? status;
}

export function brandLabel(brand: PrinterBrand): string {
  const map: Record<PrinterBrand, string> = {
    pantum:  'Pantum',
    kyocera: 'Kyocera',
    hp:      'HP',
    canon:   'Canon',
    other:   'Другой',
  };
  return map[brand] ?? brand;
}

export function supplyLabel(type: SupplyType): string {
  const map: Record<SupplyType, string> = {
    toner_black:   'Тонер (чёрный)',
    toner_cyan:    'Тонер (голубой)',
    toner_magenta: 'Тонер (пурпурный)',
    toner_yellow:  'Тонер (жёлтый)',
    drum:          'Фотобарабан',
    fuser:         'Термоузел',
    waste:         'Бункер отработки',
    other:         'Расходник',
  };
  return map[type] ?? type;
}
