# TonerScope — PROJECT STATUS

> Этот файл содержит полное состояние проекта для продолжения разработки в новом диалоге.
> Скопируй его содержимое в новый чат с Claude для бесшовного продолжения.

---

## 🧭 Что такое проект

**TonerScope** — кроссплатформенное десктопное приложение для мониторинга сетевых принтеров
в локальной сети через SNMP. Написано на **Rust + Tauri v2** (бэкенд) и **SvelteKit + SCSS**
(фронтенд). Основная задача: отслеживать уровни тонера/расходников, статусы принтеров,
автоматически сканировать подсеть, хранить историю и отправлять алерты.

**Целевые принтеры в сети:** Pantum BM5100ADN (основные), Kyocera ECOSYS, HP LaserJet, Canon iR.

**Контекст возникновения:** В сети AD (Active Directory) принтеры Pantum периодически зависают —
не печатают без ошибок, помогает рестарт спулера. Параллельная задача — мониторинг расходников.

---

## 📦 Стек технологий

| Слой | Технология | Версия |
|---|---|---|
| Desktop runtime | Tauri | 2.3.x |
| Frontend framework | SvelteKit | 2.9.x |
| Frontend language | TypeScript | 5.7.x |
| CSS preprocessor | SCSS (Sass) | 1.83.x |
| Build tool | Vite | 6.0.x |
| Backend language | Rust | stable (2021 edition) |
| SNMP | crate `snmp` | 0.2.2 |
| Database | SQLite via `rusqlite` | 0.31 (bundled) |
| Async runtime | Tokio | 1.x (full features) |
| IP networking | `ipnetwork` | 0.20 |
| Serialization | `serde` + `serde_json` | 1.x |
| UUID generation | `uuid` | 1.x (v4) |
| Date/time | `chrono` | 0.4 (serde feature) |
| Logging | `log` + `env_logger` | 0.4 / 0.11 |

---

## 🗂 Полная структура файлов проекта

```
tonerscope/
│
├── .github/
│   └── workflows/
│       ├── build.yml          ✅ CI: сборка на push/PR (Linux, macOS x86, macOS ARM, Windows)
│       └── release.yml        ✅ CD: релиз по тегу v* (.deb, .AppImage, .dmg, .msi, portable .zip)
│
├── src/                       ← SvelteKit фронтенд
│   ├── app.html               ✅ HTML точка входа (lang=ru)
│   ├── app.scss               ✅ Глобальные стили: импорт всех SCSS модулей + CSS custom properties
│   │
│   ├── lib/
│   │   ├── index.ts           ✅ Barrel export всего из lib/
│   │   │
│   │   ├── api/
│   │   │   ├── tauri.ts       ✅ Типизированная обёртка IPC. Все @tauri-apps/api — только
│   │   │   │                     динамические import() внутри функций (защита от SSR-краша).
│   │   │   │                     Экспортирует: api.{getPrinters, addPrinter, removePrinter,
│   │   │   │                     pollPrinter, getSnapshots, scanNetwork, getSettings,
│   │   │   │                     saveSettings, onPrinterUpdated, onPrinterAlert, onScanProgress}
│   │   │   │                     Типы: PrinterRecord, SupplyRecord, PrinterSnapshotRecord,
│   │   │   │                     SnapshotRecord, AppSettingsRecord, UnlistenFn,
│   │   │   │                     ScanProgressPayload, PrinterUpdatedPayload, PrinterAlertPayload
│   │   │   └── index.ts       ✅ Barrel re-export из tauri.ts
│   │   │
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   │   ├── index.ts           ✅ Barrel export
│   │   │   │   ├── Button.svelte      ✅ variants: primary/secondary/ghost/danger/outline; loading state
│   │   │   │   ├── Badge.svelte       ✅ variants: default/success/warning/error/info/neutral; dot prop
│   │   │   │   ├── Card.svelte        ✅ padding: none/sm/md/lg; hoverable/clickable/selected
│   │   │   │   ├── Input.svelte       ✅ types, label, hint, error, prefix/suffix slots
│   │   │   │   ├── Modal.svelte       ✅ sizes: sm/md/lg; footer slot; ESC закрытие
│   │   │   │   ├── ProgressBar.svelte ✅ sizes: xs/sm/md; авто-цвет от процента; animated shimmer
│   │   │   │   ├── Toast.svelte       ✅ Portal-компонент; анимации fly/fade/flip; 4 типа
│   │   │   │   └── Tooltip.svelte     ✅ positions: top/bottom/left/right; delay prop
│   │   │   │
│   │   │   ├── printer/
│   │   │   │   ├── index.ts           ✅ Barrel export
│   │   │   │   ├── PrinterCard.svelte   ✅ Карточка принтера: бренд-иконка, статус, расходники
│   │   │   │   ├── PrinterDetail.svelte ✅ Боковая панель деталей; реальный pollNow() через
│   │   │   │   │                          api.pollPrinter(ip); удаление через api.removePrinter(id)
│   │   │   │   ├── PrinterGrid.svelte   ✅ CSS grid auto-fill; skeleton loader; empty state
│   │   │   │   ├── StatusBadge.svelte   ✅ Animated ping dot для online/printing
│   │   │   │   └── TonerGauge.svelte    ✅ Название + цветной ProgressBar + алерт-бейдж
│   │   │   │
│   │   │   └── layout/
│   │   │       ├── index.ts          ✅ Barrel export
│   │   │       ├── Sidebar.svelte    ✅ Навигация; коллапс; badge кол-во; theme toggle; alert bar
│   │   │       ├── Header.svelte     ✅ Заголовок страницы; поиск; quick-status chips; actions slot
│   │   │       └── PageWrapper.svelte ✅ <main> с padding/scroll/maxWidth
│   │   │
│   │   ├── stores/
│   │   │   ├── printers.ts    ✅ MOCK удалён. Загрузка из api.getPrinters() через initPrinters().
│   │   │   │                     Подписка на printer-updated от планировщика.
│   │   │   │                     Экспортирует: printers, printerStats, scanResults, isScanning,
│   │   │   │                     selectedPrinterId, initPrinters(), destroyPrinters(), mergeSnapshot()
│   │   │   ├── settings.ts    ✅ AppSettings store; theme store — localStorage только внутри
│   │   │   │                     функций (НЕ на уровне модуля — защита от SSR-краша)
│   │   │   └── notifications.ts ✅ Toast queue store; success/error/warning/info helpers
│   │   │
│   │   ├── styles/
│   │   │   ├── _variables.scss  ✅ Дизайн-токены: Syne+JetBrains, spacing, radius, z-index
│   │   │   ├── _mixins.scss     ✅ respond-to/below, flex-center/between, card-base, scrollbar
│   │   │   ├── _animations.scss ✅ fadeIn, fadeInUp, pulse, spin, scanLine, shimmer; skeleton
│   │   │   ├── _reset.scss      ✅ Современный CSS reset
│   │   │   └── _typography.scss ✅ Google Fonts import; heading scale; text utilities
│   │   │
│   │   ├── types/
│   │   │   └── printer.ts     ✅ PrinterStatus, SupplyType, PrinterBrand, Supply, PrinterInfo,
│   │   │                         PrinterSnapshot, ScanResult, AlertRule, AppSettings, NavPage
│   │   │
│   │   └── utils/
│   │       ├── formatters.ts  ✅ formatPageCount, formatRelativeTime, statusLabel, brandLabel, supplyLabel
│   │       └── colors.ts      ✅ tonerColor, statusColor, supplyTypeColor, brandColor
│   │
│   └── routes/
│       ├── +layout.ts         ✅ ssr=false, prerender=false (static adapter для Tauri)
│       ├── +layout.svelte     ✅ Инициализация: initPrinters() + onPrinterAlert toast.
│       │                         ВАЖНО: onMount НЕ async — async-код через .then() во избежание
│       │                         потери компонентного контекста в Svelte 5 (баг ssr_context.r)
│       ├── +page.svelte       ✅ Dashboard: stat cards + PrinterGrid + PrinterDetail panel
│       ├── printers/
│       │   └── +page.svelte   ✅ Список с фильтрами по статусу/бренду/поиску; модалка добавления
│       ├── scan/
│       │   └── +page.svelte   ✅ Реальный invoke('scan_network') + listen('scan-progress').
│       │                         Прогресс-бар, лог, результаты, импорт через api.addPrinter()
│       ├── history/
│       │   └── +page.svelte   ✅ Реальные снапшоты из api.getSnapshots(printerId, 90).
│       │                         SVG sparkline: gradient fill + line. Кеш по printer.id.
│       │                         Состояния: loading / error / empty / данные
│       ├── alerts/
│       │   └── +page.svelte   ✅ Три секции: критические / заканчиваются / недоступны
│       └── settings/
│           └── +page.svelte   ✅ Загрузка из api.getSettings() в onMount.
│                                 Сохранение через api.saveSettings() (без поля theme — только SQLite-поля).
│                                 theme хранится отдельно в localStorage.
│
├── src-tauri/
│   ├── build.rs               ✅ tauri_build::build()
│   ├── Cargo.toml             ✅ Все зависимости
│   ├── tauri.conf.json        ✅ Tauri v2 конфиг; window 1200x780; bundle targets: all
│   │
│   ├── capabilities/
│   │   └── default.json       ✅ Tauri v2 capabilities: core, shell, dialog, fs, notification
│   │
│   ├── icons/                 ⚠️  ЗАГЛУШКИ (1x1 PNG) — нужно заменить реальными иконками
│   │
│   └── src/
│       ├── main.rs            ✅ windows_subsystem="windows"; вызов lib::run()
│       ├── lib.rs             ✅ Tauri Builder; setup DB + scheduler; invoke_handler со всеми
│       │                         командами включая get_snapshots
│       │
│       ├── snmp/
│       │   ├── mod.rs         ✅ pub use client::{...}
│       │   ├── oids.rs        ✅ Стандартные OID Printer MIB RFC 3805 + вендорные
│       │   └── client.rs      ✅ SnmpClient::poll() + probe()
│       │
│       ├── scanner/
│       │   ├── mod.rs         ✅ pub use network::{...}
│       │   └── network.rs     ✅ NetworkScanner::scan_subnet(); async JoinSet
│       │
│       ├── db/
│       │   ├── mod.rs         ✅ Database struct; get_snapshots(printer_id: &str, limit: i64)
│       │   │                     возвращает Vec<SnapshotRecord> с полем id: Option<i64>
│       │   └── models.rs      ✅ #[serde(rename_all = "camelCase")] на всех структурах.
│       │                         SnapshotRecord имеет id: Option<i64> с skip_serializing_if.
│       │                         AppSettings без поля theme.
│       │
│       ├── commands/
│       │   ├── mod.rs         ✅ pub mod printer/scanner/settings
│       │   ├── printer.rs     ✅ get_printers, add_printer, remove_printer, poll_printer,
│       │   │                     get_snapshots(printer_id: String, limit: Option<i64>)
│       │   ├── scanner.rs     ✅ scan_network (async, emit scan-progress events)
│       │   └── settings.rs    ✅ get_settings, save_settings
│       │
│       └── scheduler/
│           └── mod.rs         ✅ Бесконечный loop; poll_all(); emit "printer-updated" + "printer-alert"
│
├── static/
│   ├── favicon.png            ⚠️  Заглушка
│   └── favicon.svg            ✅ SVG иконка TonerScope
│
├── .gitignore                 ✅
├── package.json               ✅ scripts: dev/build/check/tauri:dev/tauri:build
├── svelte.config.js           ✅ adapter-static; kit.alias: { $lib: './src/lib' }
├── tsconfig.json              ✅ БЕЗ extends и БЕЗ baseUrl/paths (конфликтуют с SvelteKit)
├── vite.config.ts             ✅ port 1420; игнорирует src-tauri
└── README.md                  ✅
```

---

## ✅ Статус Фазы 1 — ЗАВЕРШЕНА

- ✅ Весь UI с mock-данными (6 тестовых принтеров)
- ✅ Переключение тёмной/светлой темы
- ✅ Коллапс сайдбара с сохранением в localStorage
- ✅ Фильтрация принтеров по статусу и бренду
- ✅ Добавление принтера вручную (через форму)
- ✅ Боковая панель деталей принтера
- ✅ Симуляция сканирования сети (mock)
- ✅ История с SVG sparkline графиками (mock данные)
- ✅ Страница уведомлений (критические/низкие/недоступные)
- ✅ Страница настроек (SNMP, пороги, подсети, тема)
- ✅ Toast уведомления
- ✅ Rust бэкенд компилируется (все ошибки исправлены)
- ✅ GitHub Actions: CI (build.yml) + CD (release.yml)

---

## ✅ Статус Фазы 2 — ЗАВЕРШЕНА

### Что реализовано

- ✅ **`src/lib/api/tauri.ts`** — полная типизированная IPC-обёртка. Все импорты из
  `@tauri-apps/api` — **динамические** (`await import(...)`) внутри функций. Статических
  импортов нет нигде, кроме этого файла. `browser` guard на всех методах.
- ✅ **`stores/printers.ts`** — mock удалён. `initPrinters()` / `destroyPrinters()`.
  `mergeSnapshot()` экспортирован для использования в компонентах.
- ✅ **`+layout.svelte`** — вызов `initPrinters()` и `api.onPrinterAlert()` через `.then()`
  (НЕ async onMount). Cleanup возвращается синхронно из onMount.
- ✅ **`scan/+page.svelte`** — реальный `api.scanNetwork()` + `api.onScanProgress()`.
  Импорт принтера через `api.addPrinter()`.
- ✅ **`PrinterDetail.svelte`** — `pollNow()` через `api.pollPrinter(ip)`,
  удаление через `api.removePrinter(id)`.
- ✅ **`history/+page.svelte`** — снапшоты из `api.getSnapshots()`. Парсинг `suppliesJson`
  (camelCase). Кеш по `printer.id`. SVG sparkline с gradient.
- ✅ **`settings/+page.svelte`** — загрузка из `api.getSettings()` в `onMount`,
  сохранение без поля `theme` (Rust-структура его не содержит).
- ✅ **`commands/printer.rs`** — добавлена команда `get_snapshots`.
- ✅ **`db/mod.rs`** — метод `get_snapshots(printer_id, limit: i64)` с SELECT по id.
- ✅ **`db/models.rs`** — `SnapshotRecord` с `id: Option<i64>` + `skip_serializing_if`.
  Все структуры `#[serde(rename_all = "camelCase")]`.
- ✅ **`lib.rs`** — `get_snapshots` зарегистрирован в `invoke_handler`.

### Критические решения принятые в Фазе 2

#### 1. Динамические импорты @tauri-apps/api
`@tauri-apps/api/core` и `@tauri-apps/api/event` **нельзя импортировать статически** —
SvelteKit с `prerender=true` выполняет модули в Node.js, где `window.__TAURI__` отсутствует.
Решение: все вызовы `invoke` и `listen` через `await import(...)` внутри функций `api`.

```typescript
// ✅ Правильно — динамический import внутри функции
async function inv<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!browser) throw new Error('invoke called outside browser');
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

// ❌ Неправильно — статический import на верхнем уровне модуля
import { invoke } from '@tauri-apps/api/core';
```

#### 2. onMount НЕ должен быть async (Svelte 5)
В Svelte 5 `onDestroy` теряет контекст компонента после `await` внутри `onMount`.
Решение: `onMount` синхронный, async-работа через `.then()`, cleanup возвращается из onMount.

```typescript
// ✅ Правильно
onMount(() => {
  initPrinters().then(() => { initialized = true; });
  api.onPrinterAlert(cb).then(fn => { unlistenAlert = fn; });
  return cleanup; // синхронный return
});

// ❌ Неправильно — теряет контекст, onDestroy падает
onMount(async () => {
  await initPrinters();
  onDestroy(cleanup); // <-- ssr_context.r crash
});
```

#### 3. serde rename_all = camelCase
Все Rust-структуры имеют `#[serde(rename_all = "camelCase")]`. Поля на фронте совпадают
с TS-типами напрямую: `addedManually`, `suppliesJson`, `printerId`, `pageCount`.
Дополнительный snake_case маппинг не нужен.

#### 4. AppSettings без theme
`AppSettings` в Rust не содержит поле `theme`. При `saveSettings` передаём только
SQLite-поля. `theme` хранится в `localStorage` и управляется отдельным store.

#### 5. tsconfig.json без extends и paths
`baseUrl` и `paths` в `tsconfig.json` конфликтуют с SvelteKit. Алиас `$lib` задаётся
только через `kit.alias` в `svelte.config.js`. Файл `tsconfig.json` без `extends`.

#### 6. prerender = false для Tauri
`prerender = true` заставляет SvelteKit рендерить страницы в Node.js, где нет `window`.
Для Tauri используем только `ssr = false` + `adapter-static` с `fallback: 'index.html'`.

---

## 🗺 Планы по фазам

### Фаза 3 — Страница истории с интерактивными графиками (следующий шаг)

**Цель:** полноценные интерактивные графики на основе SQLite данных.

- [ ] Компонент `src/lib/components/charts/SparklineChart.svelte`
  - Hover tooltip с датой и значением
  - Вертикальная линия-курсор при наведении
  - Чистый SVG (без сторонних библиотек)
- [ ] Фильтр периода в `history/+page.svelte`: 7 / 30 / 90 дней / всё
- [ ] Добавить Tauri команду `get_history_stats(printer_id, days)` → min/max/avg
- [ ] Прогноз расхода: линейная экстраполяция → «тонер закончится через N дней»
- [ ] Таблица под графиком с точными значениями по датам
- [ ] Экспорт в CSV

---

### Фаза 4 — Уведомления и алерты

- [ ] CRUD правил `AlertRule` — страница alerts (сейчас только отображает состояние)
- [ ] Таблица `alerts` в SQLite для хранения правил
- [ ] Desktop-уведомления через `tauri-plugin-notification`
  ```rust
  use tauri_plugin_notification::NotificationExt;
  app.notification().builder()
     .title("TonerScope: Низкий тонер")
     .body(format!("{}: {}%", printer_name, percent))
     .show()?;
  ```
- [ ] Настройка: не беспокоить в нерабочее время

---

### Фаза 5 — Иконки и брендинг

- [ ] Создать SVG иконку TonerScope 1024×1024
- [ ] Сгенерировать все форматы: `npx tauri icon path/to/icon-1024.png`
- [ ] Tray-иконка: 22×22 монохромная
- [ ] Заменить `static/favicon.png`

---

### Фаза 6 — Дополнительные функции

#### 6.1 Перезапуск Print Spooler (исходная задача)
- [ ] Tauri команда `restart_spooler(computer: String)` через WinAPI / sc.exe
- [ ] Кнопка «Перезапустить спулер» на карточке принтера (Windows only)

#### 6.2 Групповое управление
- [ ] Фильтрация по группе на dashboard (поле `group` в DB уже есть)
- [ ] Batch-опрос выбранных принтеров

#### 6.3 Экспорт отчётов
- [ ] Список принтеров в CSV
- [ ] Отчёт расхода тонера за период

#### 6.4 SNMP v3
- [ ] Поля в настройках: username, authPassword, privPassword, authProtocol, privProtocol

#### 6.5 WSD/mDNS обнаружение
- [ ] Крейт `mdns-sd` — вкладки в сканировании: SNMP / mDNS

---

## 🔌 Tauri IPC API (актуальное состояние)

### Команды (Frontend → Backend)

| Команда | Параметры | Возврат | Статус |
|---|---|---|---|
| `get_printers` | — | `Vec<PrinterRecord>` | ✅ |
| `add_printer` | `ip, name, brand, model, location?, group?` | `PrinterRecord` | ✅ |
| `remove_printer` | `id: String` | `()` | ✅ |
| `poll_printer` | `ip: String` | `PrinterSnapshot` | ✅ |
| `get_snapshots` | `printer_id: String, limit: Option<i64>` | `Vec<SnapshotRecord>` | ✅ |
| `scan_network` | `subnet: String` | `Vec<ScanResult>` | ✅ |
| `get_settings` | — | `AppSettings` | ✅ |
| `save_settings` | `settings: AppSettings` | `()` | ✅ |

### Важно: передача параметров invoke

Имена аргументов Tauri-команд передаются в **snake_case** (имена параметров Rust-функции),
независимо от `serde(rename_all)`:

```typescript
// get_snapshots принимает printer_id (snake_case — имя аргумента функции)
invoke('get_snapshots', { printer_id: printerId, limit })

// НО данные возвращаются в camelCase (serde rename_all = "camelCase")
// snap.printerId, snap.suppliesJson, snap.pageCount
```

### События (Backend → Frontend)

| Событие | Payload | Источник |
|---|---|---|
| `printer-updated` | `PrinterSnapshotRecord` (camelCase) | scheduler |
| `printer-alert` | `{ ip, supply, percent }` | scheduler |
| `scan-progress` | `{ percent, current, found }` | scanner command |

---

## 🗄 Схема БД SQLite

```sql
CREATE TABLE printers (
    id             TEXT PRIMARY KEY,     -- UUID v4
    ip             TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    brand          TEXT NOT NULL DEFAULT 'other',
    model          TEXT NOT NULL DEFAULT '',
    location       TEXT,
    grp            TEXT,                 -- group (reserved word в SQL)
    added_manually INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE snapshots (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    printer_id    TEXT NOT NULL REFERENCES printers(id) ON DELETE CASCADE,
    timestamp     TEXT NOT NULL DEFAULT (datetime('now')),
    status        TEXT NOT NULL,
    page_count    INTEGER,
    supplies_json TEXT NOT NULL DEFAULT '[]'  -- JSON: camelCase Supply[]
);
CREATE INDEX idx_snapshots_printer_ts ON snapshots(printer_id, timestamp DESC);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL               -- JSON строка для 'app_settings'
);
```

**Путь к файлу БД:**
- macOS: `~/Library/Application Support/com.tonerscope.app/tonerscope.db`
- Windows: `%APPDATA%\com.tonerscope.app\tonerscope.db`
- Linux: `~/.local/share/com.tonerscope.app/tonerscope.db`

---

## 🎨 Дизайн-система

### Переключение тем
`document.documentElement.setAttribute('data-theme', 'dark'|'light')`.
Сохраняется в `localStorage` под ключом `tonerscope-theme`.

### CSS Custom Properties (ключевые)

| Переменная | Тёмная | Светлая |
|---|---|---|
| `--bg` | `#0d0f12` | `#f4f6f9` |
| `--surface-1` | `#161a1f` | `#ffffff` |
| `--surface-2` | `#1d2229` | `#f8fafc` |
| `--accent` | `#00d4aa` | `#0099aa` |
| `--accent-muted` | `rgba(0,212,170,.12)` | `rgba(0,153,170,.10)` |
| `--border` | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--text-primary` | `#eaedf0` | `#0f1923` |
| `--text-secondary` | `#8b95a1` | `#4a5568` |
| `--text-tertiary` | `#515c68` | `#94a3b8` |

### Шрифты
- **Display/Body:** Syne (400–800) — Google Fonts
- **Mono:** JetBrains Mono (400/500/600) — IP, счётчики, OID

### Цвета статусов
```
--status-online:   #22c55e
--status-offline:  #71717a
--status-printing: #3b82f6
--status-error:    #ef4444
--status-warning:  #f59e0b
```

### Цвета расходников
```
toner_black  → var(--text-primary)
toner_cyan   → #06b6d4
toner_magenta→ #ec4899
toner_yellow → #eab308
drum         → #8b5cf6
fuser        → #f97316
waste        → var(--text-tertiary)
```

---

## 🔧 SNMP справочник

### Стандартные OID (RFC 3805)

| OID | Описание |
|---|---|
| `1.3.6.1.2.1.1.1.0` | sysDescr |
| `1.3.6.1.2.1.25.3.5.1.1.1` | hrPrinterStatus (3=idle, 4=printing, 5=warmup) |
| `1.3.6.1.2.1.43.10.2.1.4.1.1` | prtMarkerLifeCount — страниц |
| `1.3.6.1.2.1.43.11.1.1.6.1.{i}` | prtMarkerSuppliesDescription[i] |
| `1.3.6.1.2.1.43.11.1.1.4.1.{i}` | prtMarkerSuppliesType[i] (3=toner, 16=drum) |
| `1.3.6.1.2.1.43.11.1.1.8.1.{i}` | prtMarkerSuppliesMaxCapacity[i] |
| `1.3.6.1.2.1.43.11.1.1.9.1.{i}` | prtMarkerSuppliesLevel[i] |
| `1.3.6.1.2.1.43.12.1.1.4.1.{i}` | prtMarkerColorantValue[i] |

### Вендорные OID

| Бренд | OID | Описание |
|---|---|---|
| Pantum | `1.3.6.1.4.1.40945.1.1.2.15.0` | Счётчик страниц |
| Kyocera | `1.3.6.1.4.1.1347.42.2.1.1.4.1.1` | Счётчик страниц |
| Kyocera | `1.3.6.1.4.1.1347.42.3.10.5.0` | Остаток барабана (%) |
| HP | `1.3.6.1.4.1.11.2.3.9.4.2.1.4.1.5.0` | Счётчик страниц |
| Canon | `1.3.6.1.4.1.1602.1.1.1.1.10.0` | Счётчик страниц |

### SNMP крейт API
```
snmp = "0.2.2"
SyncSession::new(addr, &[u8], Option<Duration>, i32) -> io::Result<SyncSession>
session.get(&[u32]) -> SnmpResult<SnmpPdu>
pdu.varbinds — итератор (OID, Value)

Value enum (НЕТ Gauge32 — используй Unsigned32):
  Value::Integer(i64)
  Value::OctetString(&[u8])
  Value::Counter32(u32)
  Value::Unsigned32(u32)   ← покрывает Gauge32
  Value::Counter64(u64)
```

---

## 🚀 Быстрый старт

```bash
npm install --legacy-peer-deps
npm run tauri:dev      # полный запуск
npm run dev            # только фронтенд (без Tauri)
npm run tauri:build    # сборка релиза
```

### Linux зависимости
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libssl-dev
```

---

## 💬 Контекст для нового диалога

> Продолжаем разработку TonerScope — приложения для мониторинга сетевых принтеров
> на Tauri v2 + SvelteKit + SCSS. Фазы 1 и 2 завершены. Нужно реализовать Фазу 3:
> интерактивные графики истории тонера с hover-tooltip, фильтрацией по периоду
> и прогнозом расхода. Полное состояние проекта в файле PROJECT_STATUS.md.

---

*Последнее обновление: Фаза 2 завершена. Фронтенд запускается в Tauri без ошибок.*
*Критические баги исправлены: SSR-краш @tauri-apps/api, async onMount в Svelte 5.*
