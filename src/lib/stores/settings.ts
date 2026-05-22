// src/lib/stores/settings.ts

import { writable, derived } from 'svelte/store';
import type { AppSettings } from '$lib/types/printer';

const DEFAULT_SETTINGS: AppSettings = {
  pollIntervalMinutes:    5,
  lowTonerThreshold:      20,
  criticalTonerThreshold: 10,
  snmpCommunity:          'public',
  snmpTimeout:            3,
  snmpRetries:            2,
  theme:                  'system',
  subnets:                ['192.168.1.0/24'],
};

function createSettingsStore() {
  const { subscribe, set, update } = writable<AppSettings>(DEFAULT_SETTINGS);

  return {
    subscribe,
    set,
    update,
    patch: (partial: Partial<AppSettings>) =>
      update(s => ({ ...s, ...partial })),
    reset: () => set(DEFAULT_SETTINGS),
  };
}

export const settings = createSettingsStore();

// ─── Theme Store ──────────────────────────────────────────────────────────────
export type Theme = 'dark' | 'light' | 'system';
export type ResolvedTheme = 'dark' | 'light';

function getSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function createThemeStore() {
  // НЕ обращаемся к localStorage на уровне модуля — только внутри функций
  const { subscribe, set, update } = writable<Theme>('system');

  function applyTheme(t: Theme) {
    const resolved: ResolvedTheme = t === 'system' ? getSystemTheme() : t;
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', resolved);
    }
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('tonerscope-theme', t);
    }
  }

  return {
    subscribe,
    set: (t: Theme) => {
      set(t);
      applyTheme(t);
    },
    toggle: () => update(t => {
      const next = t === 'dark' ? 'light' : 'dark';
      applyTheme(next);
      return next;
    }),
    // Вызывается только из onMount в +layout.svelte — безопасно
    init: () => {
      const stored = typeof localStorage !== 'undefined'
        ? (localStorage.getItem('tonerscope-theme') as Theme | null)
        : null;
      const t = stored ?? 'system';
      set(t);
      applyTheme(t);

      if (typeof window !== 'undefined') {
        window.matchMedia('(prefers-color-scheme: dark)')
          .addEventListener('change', () => {
            update(cur => { applyTheme(cur); return cur; });
          });
      }
    },
  };
}

export const theme = createThemeStore();

export const resolvedTheme = derived(theme, ($theme): ResolvedTheme =>
  $theme === 'system' ? getSystemTheme() : $theme
);
