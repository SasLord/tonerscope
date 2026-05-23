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
│   │   │   │                     pollPrinter, getSnapshots, getHistoryStats, scanNetwork,
│   │   │   │                     getSettings, saveSettings, getAlertRules, saveAlertRule,
│   │   │   │                     deleteAlertRule, onPrinterUpdated, onPrinterAlert, onScanProgress}
│   │   │   │                     Типы: PrinterRecord, SupplyRecord, PrinterSnapshotRecord,
│   │   │   │                     SnapshotRecord, HistoryStatsRecord, SupplyStatRecord,
│   │   │   │                     AppSettingsRecord, AlertRuleRecord, UnlistenFn,
│   │   │   │                     ScanProgressPayload, PrinterUpdatedPayload, PrinterAlertPayload
│   │   │   └── index.ts       ✅ Barrel re-export из tauri.ts (включая AlertRuleRecord)
│   │   │
│   │   ├── components/
│   │   │   ├── ui/
│   │   │   │   ├── index.ts           ✅ Barrel export
│   │   │   │   ├── Button.svelte      ✅ variants: primary/secondary/ghost/danger/outline; loading state
│   │   │   │   │                         ВАЖНО: только дефолтный <slot />, нет слота prefix/suffix
│   │   │   │   ├── Badge.svelte       ✅ variants: default/success/warning/error/info/neutral; dot prop
│   │   │   │   ├── Card.svelte        ✅ padding: none/sm/md/lg; hoverable/clickable/selected
│   │   │   │   ├── Input.svelte       ✅ types, label, hint, error, prefix/suffix slots
│   │   │   │   ├── Modal.svelte       ✅ sizes: sm/md/lg; footer slot; ESC закрытие
│   │   │   │   ├── ProgressBar.svelte ✅ sizes: xs/sm/md; авто-цвет от процента; animated shimmer
│   │   │   │   ├── Toast.svelte       ✅ Portal-компонент; анимации fly/fade/flip; 4 типа
│   │   │   │   └── Tooltip.svelte     ✅ positions: top/bottom/left/right; delay prop
│   │   │   │
│   │   │   ├── charts/
│   │   │   │   ├── index.ts           ✅ Barrel export
│   │   │   │   └── SparklineChart.svelte ✅ Интерактивный SVG-график:
│   │   │   │                               - hover-tooltip (дата + процент)
│   │   │   │                               - вертикальный курсор-линия
│   │   │   │                               - фоновые зоны критического (<10%) и низкого (<20%) уровня
│   │   │   │                               - горизонтальные guideline на 20/50/80%
│   │   │   │                               - пунктирная линия прогноза (МНК-экстраполяция)
│   │   │   │                               - бейдж «Прогноз: ~N дн.» с пульсацией при критическом
│   │   │   │                               - анимация появления линии (stroke-dashoffset)
│   │   │   │                               - Props: points, color, supplyType, forecastDays
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
│   │   │   └── notifications.ts ✅ Toast queue store. Паттерн использования:
│   │   │                          import { notifications } from '$lib/stores/notifications'
│   │   │                          notifications.success(title, message?)
│   │   │                          notifications.error(title, message?)
│   │   │                          notifications.warning(title, message?)
│   │   │                          notifications.info(title, message?)
│   │   │                          ВАЖНО: НЕТ именованных экспортов success/error/warning/info —
│   │   │                          только через объект notifications.
│   │   │
│   │   ├── styles/
│   │   │   ├── _variables.scss  ✅ Дизайн-токены: Syne+JetBrains, spacing, radius, z-index
│   │   │   │                       ВАЖНО: нет переменной $font-weight-normal.
│   │   │   │                       Доступные font-weight: $font-weight-regular, $font-weight-medium,
│   │   │   │                       $font-weight-semibold, $font-weight-bold, $font-weight-black
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
│       │   └── +page.svelte   ✅ Реальные снапшоты из api.getSnapshots().
│       │                         Интерактивные графики SparklineChart. Фильтр периода (7/30/90/all).
│       │                         Сводная строка статистики. Прогноз расхода (МНК).
│       │                         Раскрывающаяся таблица снапшотов с Δ. Экспорт в CSV.
│       │                         Кеш снапшотов по printer.id — смена периода без повторных запросов.
│       ├── alerts/
│       │   └── +page.svelte   ✅ ПЕРЕРАБОТАНА В ФАЗЕ 4. Две секции:
│       │                         1. Текущее состояние (критические / заканчиваются / недоступны)
│       │                         2. CRUD правил уведомлений: таблица правил, toggle enabled/desktop,
│       │                            модалка добавления/редактирования с range-слайдером порога,
│       │                            выбор принтера и типа расходника, кнопки удаления
│       └── settings/
│           └── +page.svelte   ✅ Загрузка из api.getSettings() в onMount.
│                                 Сохранение через api.saveSettings() (без поля theme — только SQLite-поля).
│                                 theme хранится отдельно в localStorage.
│
├── src-tauri/
│   ├── build.rs               ✅ tauri_build::build()
│   ├── Cargo.toml             ✅ Все зависимости включая tauri-plugin-notification = "2"
│   ├── tauri.conf.json        ✅ Tauri v2 конфиг; window 1200x780; bundle targets: all
│   │
│   ├── capabilities/
│   │   └── default.json       ✅ Tauri v2 capabilities: core, shell, dialog, fs, notification
│   │
│   ├── icons/                 ✅ ФАЗА 5: SVG-иконки созданы, PNG генерируются через npx tauri icon
│   │   ├── tonerscope-1024.svg  ✅ Мастер-иконка 1024×1024 (прицел + картридж + уровень тонера)
│   │   ├── tray-icon.svg        ✅ Монохромная tray-иконка 22×22 (белая для тёмной системной панели)
│   │   ├── 32x32.png            ⚠️  Генерировать: npx tauri icon src-tauri/icons/tonerscope-1024.png
│   │   ├── 128x128.png          ⚠️  —//—
│   │   ├── 128x128@2x.png       ⚠️  —//—
│   │   ├── icon.icns            ⚠️  —//—
│   │   └── icon.ico             ⚠️  —//—
│   │
│   └── src/
│       ├── main.rs            ✅ windows_subsystem="windows"; вызов lib::run()
│       ├── lib.rs             ✅ Tauri Builder; setup DB + scheduler; invoke_handler со всеми
│       │                         командами включая get_alert_rules, save_alert_rule, delete_alert_rule
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
│       │   ├── mod.rs         ✅ Database struct; методы:
│       │   │                     get_snapshots(printer_id, limit: i64)
│       │   │                     get_history_stats(printer_id, period_days: i64) — МНК-прогноз
│       │   │                     get_alert_rules() → Vec<AlertRule>
│       │   │                     save_alert_rule(rule: &AlertRule) — upsert по id
│       │   │                     delete_alert_rule(id: &str)
│       │   │                     Вспомогательная fn compute_forecast_days(pts) — МНК по 30 точкам.
│       │   └── models.rs      ✅ #[serde(rename_all = "camelCase")] на всех структурах.
│       │                         SnapshotRecord: id: Option<i64> с skip_serializing_if.
│       │                         AppSettings без поля theme.
│       │                         SupplyStatRecord + HistoryStatsRecord (Фаза 3).
│       │                         AlertRule: id, printer_id, supply_type, threshold, enabled,
│       │                         notify_desktop (Фаза 4).
│       │
│       ├── commands/
│       │   ├── mod.rs         ✅ pub mod alerts/printer/scanner/settings
│       │   ├── alerts.rs      ✅ НОВЫЙ ФАЙЛ (Фаза 4):
│       │   │                     get_alert_rules, save_alert_rule(rule: AlertRule),
│       │   │                     delete_alert_rule(id: String)
│       │   ├── printer.rs     ✅ get_printers, add_printer, remove_printer, poll_printer,
│       │   │                     get_snapshots(printer_id, limit: Option<i64>),
│       │   │                     get_history_stats(printer_id, period_days: Option<i64>)
│       │   ├── scanner.rs     ✅ scan_network (async, emit scan-progress events)
│       │   └── settings.rs    ✅ get_settings, save_settings
│       │
│       └── scheduler/
│           └── mod.rs         ✅ Бесконечный loop; poll_all(); emit "printer-updated" +
│                                 "printer-alert". В Фазе 4 добавлено:
│                                 — загружает alert_rules из БД перед каждым циклом опроса
│                                 — для каждого расходника проверяет совпадение по printer_id,
│                                   supply_type и threshold
│                                 — вызывает send_desktop_notification() через
│                                   tauri_plugin_notification::NotificationExt
│                                 — percent: u8 → i32::from(supply.percent) при сравнении
│
├── static/
│   ├── favicon.png            ⚠️  Генерировать из favicon.svg (32×32 PNG через inkscape/rsvg)
│   └── favicon.svg            ✅ ФАЗА 5: адаптивный SVG (prefers-color-scheme); прицел+картридж
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

- ✅ **`src/lib/api/tauri.ts`** — полная типизированная IPC-обёртка. Все импорты из
  `@tauri-apps/api` — **динамические** (`await import(...)`) внутри функций. `browser` guard.
- ✅ **`stores/printers.ts`** — mock удалён. `initPrinters()` / `destroyPrinters()`.
- ✅ **`+layout.svelte`** — вызов `initPrinters()` и `api.onPrinterAlert()` через `.then()`.
- ✅ **`scan/+page.svelte`** — реальный `api.scanNetwork()` + `api.onScanProgress()`.
- ✅ **`PrinterDetail.svelte`** — `pollNow()` через `api.pollPrinter(ip)`.
- ✅ **`history/+page.svelte`** — снапшоты из `api.getSnapshots()`.
- ✅ **`settings/+page.svelte`** — загрузка/сохранение без поля `theme`.
- ✅ **`commands/printer.rs`** — команда `get_snapshots`.
- ✅ **`db/mod.rs`** — метод `get_snapshots`.
- ✅ **`lib.rs`** — `get_snapshots` зарегистрирован в `invoke_handler`.

---

## ✅ Статус Фазы 3 — ЗАВЕРШЕНА

- ✅ **`SparklineChart.svelte`** — интерактивный SVG-график без сторонних библиотек.
  Hover-курсор, tooltip, зоны критического уровня, guideline, МНК-прогноз, анимация.
- ✅ **`history/+page.svelte`** — фильтр периода 7/30/90/all, сетка карточек расходников,
  сводная строка статистики, раскрывающаяся таблица снапшотов с Δ, экспорт CSV.
- ✅ **`api/tauri.ts`** — `getHistoryStats`, типы `SupplyStatRecord` и `HistoryStatsRecord`.
- ✅ **`db/models.rs`** — структуры `SupplyStatRecord`, `HistoryStatsRecord`.
- ✅ **`db/mod.rs`** — метод `get_history_stats` с МНК-прогнозом.
- ✅ **`commands/printer.rs`** — команда `get_history_stats`.
- ✅ **`lib.rs`** — `get_history_stats` зарегистрирован.

---

## ✅ Статус Фазы 4 — ЗАВЕРШЕНА

### Что реализовано

- ✅ **`src-tauri/src/db/models.rs`** — добавлена структура:
  ```rust
  pub struct AlertRule {
      pub id:             String,
      pub printer_id:     String,  // UUID или "all"
      pub supply_type:    String,  // тип расходника или "any"
      pub threshold:      i64,     // порог (%)
      pub enabled:        bool,
      pub notify_desktop: bool,
  }
  ```

- ✅ **`src-tauri/src/db/mod.rs`** — таблица `alert_rules` в `init_schema()` (CREATE IF NOT EXISTS).
  Три новых метода: `get_alert_rules()`, `save_alert_rule()` (upsert), `delete_alert_rule()`.

- ✅ **`src-tauri/src/commands/alerts.rs`** — новый файл:
  ```rust
  #[tauri::command] pub fn get_alert_rules(...)  -> Result<Vec<AlertRule>, String>
  #[tauri::command] pub fn save_alert_rule(rule: AlertRule) -> Result<(), String>
  #[tauri::command] pub fn delete_alert_rule(id: String) -> Result<(), String>
  ```

- ✅ **`src-tauri/src/commands/mod.rs`** — добавлен `pub mod alerts`.

- ✅ **`src-tauri/src/lib.rs`** — три новые команды зарегистрированы в `invoke_handler`.

- ✅ **`src-tauri/src/scheduler/mod.rs`** — переработан `poll_all()`:
  - Загружает `alert_rules` из БД в начале каждого цикла.
  - Для каждого расходника снапшота проверяет все активные правила:
    `r.printer_id == "all" || == printer_id`, `r.supply_type == "any" || == supply_type`,
    `i32::from(supply.percent) <= r.threshold as i32`.
  - При совпадении вызывает `send_desktop_notification()` через
    `tauri_plugin_notification::NotificationExt`.
  - Исправлен тип: `supply.percent: u8` → `i32::from(supply.percent)` при сравнении и передаче.

- ✅ **`src/lib/api/tauri.ts`** — добавлен интерфейс `AlertRuleRecord` и три метода:
  ```typescript
  getAlertRules(): Promise<AlertRuleRecord[]>
  saveAlertRule(rule: AlertRuleRecord): Promise<void>   // invoke('save_alert_rule', { rule })
  deleteAlertRule(id: string): Promise<void>             // invoke('delete_alert_rule', { id })
  ```

- ✅ **`src/lib/api/index.ts`** — добавлен реэкспорт `AlertRuleRecord`.

- ✅ **`src/routes/alerts/+page.svelte`** — полностью переработана:
  - **Секция 1 — Текущее состояние:** критические / заканчиваются / недоступны (компактный вид).
  - **Секция 2 — Правила уведомлений:**
    - Таблица правил: 6 колонок (принтер, расходник, порог, desktop-toggle, enabled-toggle, actions).
    - Toggle `enabled` через `toggle-switch` компонент прямо в строке.
    - Toggle `notifyDesktop` через icon-кнопку (bell icon).
    - Кнопки редактирования (pencil) и удаления (trash) для каждой строки.
    - Empty state с иллюстрацией и кнопкой «Создать первое правило».
    - Skeleton loader при загрузке.
    - **Модалка добавления/редактирования:**
      - Select принтера (all + список из store).
      - Select типа расходника (any + 7 типов).
      - Range-слайдер порога 1–99% с live-цветом и метками Крит/Низко/Норма.
      - Чекбоксы: Desktop-уведомления + Правило активно.
    - UUID генерируется на фронте через `crypto.randomUUID()`.

### Исправленные баги Фазы 4

- `$font-weight-normal` → не существует, убран (использован дефолтный вес).
- `import { success, error }` → не именованные экспорты; заменено на `notifications.success()`.
- `<svg slot="prefix">` в Button → Button не имеет слота prefix; SVG инлайном в дефолтный slot.
- `supply.percent: u8` vs `r.threshold as i32` → исправлено через `i32::from(supply.percent)`.

---

## ✅ Статус Фазы 5 — ЗАВЕРШЕНА

### Что реализовано

- ✅ **`static/favicon.svg`** — адаптивная SVG-иконка для браузерной вкладки.
  Концепт: прицел (scope) + тонер-картридж с полоской уровня.
  Адаптируется к теме ОС через `@media (prefers-color-scheme)`:
  тёмная тема: акцент `#00d4aa` / фон `#0d0f12`;
  светлая тема: акцент `#0099aa` / фон `#f4f6f9`.

- ✅ **`src-tauri/icons/tonerscope-1024.svg`** — мастер-иконка 1024×1024.
  Скруглённый фон `rx="200"` под все платформы. Три кольца (декоративное / вспомогательное /
  прицельное), четыре засечки по 45°, крестовина, тонер-картридж с полоской уровня 72%
  и рядом точек-индикаторов под ней.

- ✅ **`src-tauri/icons/tray-icon.svg`** — монохромная tray-иконка 22×22.
  По умолчанию белая (для тёмной системной панели). Для светлой — заменить `#ffffff` на `#000000`.

- ⚠️ **PNG-иконки нужно сгенерировать** (делается один раз командой):
  ```bash
  # 1. Конвертировать SVG → PNG (любым из способов):
  inkscape src-tauri/icons/tonerscope-1024.svg \
    --export-type=png \
    --export-filename=src-tauri/icons/tonerscope-1024.png \
    --export-width=1024 --export-height=1024

  # 2. Сгенерировать все форматы для Tauri:
  npx tauri icon src-tauri/icons/tonerscope-1024.png
  # Создаст: 32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico
  ```

- ⚠️ **favicon.png** — также нужно сгенерировать из favicon.svg (32×32).

- ✅ `tauri.conf.json` → `bundle.icon` уже прописывает правильные пути — ничего менять не нужно.

---

## 🗺 Планы по фазам

### Фаза 6 — Дополнительные функции

#### 6.1 Перезапуск Print Spooler (исходная задача — приоритет)
- [ ] Tauri команда `restart_spooler(computer: String)` через `sc.exe stop/start spooler`
      или WinAPI `ControlService` / `StartService`
- [ ] Кнопка «Перезапустить спулер» на карточке и в панели деталей принтера (Windows only)
- [ ] Статус операции через Toast

#### 6.2 Групповое управление
- [ ] Фильтрация по группе на dashboard (поле `grp` в DB уже есть)
- [ ] Batch-опрос выбранных принтеров (чекбоксы в PrinterGrid)

#### 6.3 Экспорт отчётов
- [ ] Отчёт расхода тонера за период (частично реализован через CSV в Фазе 3)
- [ ] Список всех принтеров в CSV (с текущими уровнями)

#### 6.4 SNMP v3
- [ ] Поля в настройках: username, authPassword, privPassword, authProtocol, privProtocol
- [ ] Поддержка в `snmp/client.rs`

#### 6.5 WSD/mDNS обнаружение
- [ ] Крейт `mdns-sd` — вкладки в сканировании: SNMP / mDNS

#### 6.6 Поддержка USB и расшаренных принтеров (Windows only, через WMI)

**Контекст:** TonerScope сейчас работает только с принтерами, у которых есть собственный IP
в сети (прямое подключение по Ethernet/Wi-Fi). Принтер, подключённый по USB к компьютеру
и расшаренный через Windows, IP не имеет и по SNMP недоступен. Для таких принтеров
нужен другой механизм — WMI (Windows Management Instrumentation).

**Ограничения WMI-подхода:**
- Windows only (на Linux/macOS не работает — там USB-принтеры через CUPS)
- Уровень тонера и расходники отдаются только если драйвер принтера их поддерживает
  (Pantum, HP, Kyocera — обычно поддерживают; дешёвые OEM-драйверы — нет)
- Принтер должен быть установлен на том ПК, где запущен TonerScope, или на удалённом
  хосте, доступном по WMI (требует прав администратора и открытого DCOM)

**Что нужно реализовать:**

Rust-бэкенд:
- [ ] Новый модуль `src-tauri/src/wmi/mod.rs` — опрос через WMI
- [ ] Зависимость `wmi = "0.13"` в `Cargo.toml` (только под `[target.'cfg(windows)'.dependencies]`)
- [ ] WMI-запрос `Win32_Printer` — список установленных принтеров (имя, статус, порт, shared)
- [ ] WMI-запрос `Win32_PnPEntity` или `CIM_Printer` — данные о расходниках (если драйвер поддерживает)
- [ ] Tauri-команда `get_local_printers()` → `Vec<LocalPrinterRecord>` (Windows only, на других ОС — пустой массив)
- [ ] Tauri-команда `poll_local_printer(printer_name: String)` → `LocalPrinterSnapshot`
- [ ] Интеграция в планировщик: опрашивать локальные принтеры наравне со SNMP-принтерами

БД:
- [ ] Добавить поле `connection_type TEXT NOT NULL DEFAULT 'snmp'` в таблицу `printers`
      (значения: `'snmp'` / `'wmi'`)
- [ ] Добавить поле `host_name TEXT` — имя Windows-хоста для WMI (пусто для SNMP-принтеров)
- [ ] Добавить поле `printer_name TEXT` — системное имя принтера в Windows (для WMI)
- [ ] Миграция схемы: `ALTER TABLE printers ADD COLUMN ...` с `IF NOT EXISTS` guard

Фронтенд:
- [ ] Расширить `PrinterRecord` и `AppSettingsRecord` новыми полями
- [ ] Добавить в `scan/+page.svelte` третью вкладку **«Локальные (USB/расшаренные)»**
      с кнопкой «Найти на этом ПК» → `invoke('get_local_printers')`
- [ ] В модалке добавления принтера (`printers/+page.svelte`) — переключатель типа:
      **SNMP (по IP)** / **WMI (локальный/расшаренный)**; при WMI скрывать поле IP,
      показывать поле «Имя хоста» и выпадающий список обнаруженных принтеров
- [ ] В `PrinterCard.svelte` и `PrinterDetail.svelte` — иконка/бейдж типа подключения
      (сетевой / USB) рядом с именем принтера
- [ ] В `StatusBadge.svelte` — учитывать WMI-статусы (`Idle` / `Printing` / `Error` / `Offline`)

**WMI-структуры (Rust):**
```rust
// src-tauri/src/wmi/mod.rs
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32Printer {
    name: String,
    printer_status: u32,   // 3=Idle, 4=Printing, 5=Warmup, 6=Stopped
    work_offline: bool,
    shared: bool,
    share_name: Option<String>,
    port_name: Option<String>,
    driver_name: Option<String>,
    detected_error_state: u32,
}
```

**WMI-статусы PrinterStatus:**
```
1=Other, 2=Unknown, 3=Idle, 4=Printing, 5=Warmup,
6=Stopped Printing, 7=Offline
```

**Новые Tauri-команды:**
```
get_local_printers()                        → Vec<LocalPrinterRecord>
poll_local_printer(printer_name: String)    → LocalPrinterSnapshot
```

**Примечание по Linux/macOS:** на этих платформах USB-принтеры управляются через CUPS.
Поддержка CUPS (`lpstat -p`, `lpstat -s`) — отдельная подзадача, в текущий план не включена.

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
| `get_history_stats` | `printer_id: String, period_days: Option<i64>` | `HistoryStatsRecord` | ✅ |
| `scan_network` | `subnet: String` | `Vec<ScanResult>` | ✅ |
| `get_settings` | — | `AppSettings` | ✅ |
| `save_settings` | `settings: AppSettings` | `()` | ✅ |
| `get_alert_rules` | — | `Vec<AlertRule>` | ✅ |
| `save_alert_rule` | `rule: AlertRule` | `()` | ✅ |
| `delete_alert_rule` | `id: String` | `()` | ✅ |

### Важно: передача параметров invoke

Имена аргументов Tauri-команд передаются в **snake_case** (имена параметров Rust-функции),
независимо от `serde(rename_all)`:

```typescript
invoke('get_snapshots', { printer_id: printerId, limit })
invoke('get_history_stats', { printer_id: printerId, period_days: 30 })
invoke('save_alert_rule', { rule })   // rule — объект AlertRuleRecord (camelCase поля)
invoke('delete_alert_rule', { id })

// Данные возвращаются в camelCase (serde rename_all = "camelCase")
// snap.printerId, snap.suppliesJson, snap.pageCount
// rule.printerId, rule.supplyType, rule.notifyDesktop
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

CREATE TABLE alert_rules (
    id             TEXT PRIMARY KEY,   -- UUID v4, генерируется на фронте
    printer_id     TEXT NOT NULL DEFAULT 'all',   -- UUID или "all"
    supply_type    TEXT NOT NULL DEFAULT 'any',   -- тип или "any"
    threshold      INTEGER NOT NULL DEFAULT 20,   -- порог (%)
    enabled        INTEGER NOT NULL DEFAULT 1,    -- 0/1
    notify_desktop INTEGER NOT NULL DEFAULT 1     -- 0/1
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
| `--surface-3` | `#242b33` | `#f0f4f8` |
| `--accent` | `#00d4aa` | `#0099aa` |
| `--accent-muted` | `rgba(0,212,170,.12)` | `rgba(0,153,170,.10)` |
| `--border` | `rgba(255,255,255,.07)` | `rgba(0,0,0,.08)` |
| `--border-hover` | `rgba(255,255,255,.15)` | `rgba(0,0,0,.15)` |
| `--text-primary` | `#eaedf0` | `#0f1923` |
| `--text-secondary` | `#8b95a1` | `#4a5568` |
| `--text-tertiary` | `#515c68` | `#94a3b8` |
| `--nav-hover-bg` | `rgba(255,255,255,.04)` | `rgba(0,0,0,.04)` |
| `--nav-active-bg` | `rgba(0,212,170,.08)` | `rgba(0,153,170,.08)` |

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
--status-unknown:  #a1a1aa
```

### Цвета расходников (supplyTypeColor)
```
toner_black   → var(--text-primary)  ≈ #e4e4e7
toner_cyan    → #06b6d4
toner_magenta → #ec4899
toner_yellow  → #eab308
drum          → #8b5cf6
fuser         → #f97316
waste         → var(--text-tertiary)
other         → var(--text-secondary)
```

### Цвета уровня тонера (tonerColor)
```
≤ 10% → var(--gauge-crit)  = --status-error  = #ef4444
≤ 20% → var(--gauge-low)   = --status-warning = #f59e0b
> 20% → var(--gauge-ok)    = --status-online  = #22c55e
```

### SCSS-переменные (ключевые)
```scss
$font-mono:    'JetBrains Mono', monospace;
$font-display: 'Syne', sans-serif;

// Spacing
$space-1: 0.25rem;  $space-2: 0.5rem;   $space-3: 0.75rem;
$space-4: 1rem;     $space-6: 1.5rem;   $space-8: 2rem;

// Radius
$radius-sm: 4px;  $radius-md: 8px;  $radius-lg: 12px;  $radius-xl: 16px;  $radius-full: 9999px;

// Font weights (ВАЖНО: нет $font-weight-normal!)
$font-weight-regular:  400;
$font-weight-medium:   500;
$font-weight-semibold: 600;
$font-weight-bold:     700;
$font-weight-black:    800;

// Transitions
$transition-fast: 100ms ease;
$transition-base: 200ms ease;
$transition-slow: 350ms cubic-bezier(0.4, 0, 0.2, 1);

// Z-index
$z-dropdown: 100;  $z-overlay: 300;  $z-modal: 400;  $z-toast: 500;
```

### SCSS-миксины (доступные)
```scss
@include m.respond-to('md')       // min-width
@include m.respond-below('md')    // max-width
@include m.flex-center            // d:flex + align+justify center
@include m.flex-between           // d:flex + space-between
@include m.flex-start             // d:flex + align center + justify start
@include m.card-base              // surface-1 + border + radius-lg + transition
@include m.card-hover             // &:hover border + shadow
@include m.focus-ring             // &:focus-visible outline accent
@include m.custom-scrollbar(4px)  // thin scrollbar
@include m.truncate               // overflow ellipsis
@include m.glass(12px, 0.08)      // backdrop-filter blur
@include m.text-mono($size)
@include m.text-label             // mono + uppercase + letter-spacing
```

### Компонент Button — важно
```svelte
<!-- Button имеет ТОЛЬКО дефолтный <slot />, нет именованных слотов prefix/suffix -->
<!-- SVG-иконки инлайном внутри тега Button: -->
<Button variant="primary" on:click={handler}>
  <svg width="14" height="14" ...>...</svg>
  Текст кнопки
</Button>
```

### Компонент notifications — важно
```typescript
// ТОЛЬКО через объект notifications, НЕТ именованных экспортов:
import { notifications } from '$lib/stores/notifications';
notifications.success('Заголовок', 'опциональное сообщение');
notifications.error('Заголовок');
notifications.warning('Заголовок');
notifications.info('Заголовок');
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
> на Tauri v2 + SvelteKit + SCSS. Фазы 1, 2, 3, 4 и 5 завершены.
> Следующий шаг — Фаза 6: дополнительные функции.
> Приоритет: Фаза 6.1 — перезапуск Print Spooler (исходная задача проекта).
> В плане также Фаза 6.6 — поддержка USB/расшаренных принтеров через WMI (Windows only):
> принтеры без собственного IP, подключённые по USB и расшаренные через Windows,
> не видны по SNMP — для них нужен отдельный WMI-механизм.
> Полное состояние проекта в файле PROJECT_STATUS.md.

---

## ⚠️ Критические решения (не нарушать)

1. **`@tauri-apps/api` — только динамические импорты** внутри функций `api`. Никаких статических
   импортов на уровне модуля — SSR-краш в Node.js.

2. **`onMount` НЕ async** в Svelte 5. Async-работа через `.then()`. Cleanup возвращается
   синхронно из `onMount(() => { ...; return cleanup; })`.

3. **`serde(rename_all = "camelCase")`** на всех Rust-структурах. Параметры Tauri-команд —
   snake_case (имена аргументов функции). Возвращаемые поля — camelCase.

4. **`AppSettings` без `theme`** в Rust. Поле `theme` только в localStorage.

5. **`tsconfig.json` без `extends` и `paths`**. Алиас `$lib` — только через `kit.alias`.

6. **`prerender = false`** в `+layout.ts`. Только `ssr = false`.

7. **`--surface-3`** — используется в SparklineChart для tooltip. CSS-переменная определена
   в `app.scss` для обеих тем.

8. **`Button.svelte` — нет слота `prefix`**. SVG-иконки инлайном внутри дефолтного `<slot>`.

9. **`notifications` — нет именованных экспортов** `success`/`error`/`warning`/`info`.
   Только `notifications.success()` / `notifications.error()` и т.д.

10. **`$font-weight-normal` не существует** в `_variables.scss`. Использовать
    `$font-weight-regular` (400) или просто не указывать font-weight (браузерный дефолт).

11. **`supply.percent` имеет тип `u8`** в Rust. При сравнении с `i32` или `i64` использовать
    явное приведение: `i32::from(supply.percent)` или `supply.percent.into()`.

---

*Последнее обновление: Фаза 5 завершена. В план добавлена Фаза 6.6.*
*Реализованы SVG-иконки: favicon.svg (адаптивный, светлая/тёмная тема), tonerscope-1024.svg*
*(мастер для npx tauri icon), tray-icon.svg (монохромная 22×22). PNG-иконки генерируются*
*командой `npx tauri icon` из конвертированного PNG мастера.*
*Фаза 6.6: добавлена поддержка USB/расшаренных принтеров через WMI (Windows only) —*
*план включает новый модуль wmi/, расширение схемы БД, две новые Tauri-команды и UI.*
