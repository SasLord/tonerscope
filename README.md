# 🖨 TonerScope

**Мониторинг принтеров в локальной сети** — кроссплатформенное десктопное приложение на Tauri + SvelteKit для отслеживания уровня тонера, статусов и расходников сетевых принтеров через SNMP.

![TonerScope](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

---

## ✨ Возможности

- **SNMP мониторинг** — поддержка Pantum, Kyocera, HP, Canon и любых принтеров с SNMP v1/v2c
- **Сканирование сети** — автопоиск принтеров в подсети (CIDR)
- **Уровни расходников** — тонер (CMYK), фотобарабан, термоузел, бункер отработки
- **Счётчик страниц** — история печати
- **Уведомления** — алерты при низком уровне тонера
- **История** — графики изменения уровней расходников (SQLite)
- **Тёмная и светлая темы**
- **Два языка интерфейса** — русский (основной)

---

## 🚀 Установка

### Скачать готовый дистрибутив

Перейдите в [Releases](../../releases) и скачайте файл для вашей ОС:

| ОС | Файл |
|---|---|
| Windows (установщик) | `TonerScope_x.x.x_x64-setup.exe` |
| Windows (portable) | `TonerScope-portable-windows.zip` |
| Windows (MSI) | `TonerScope_x.x.x_x64_en-US.msi` |
| macOS (Apple Silicon) | `TonerScope_x.x.x_aarch64.dmg` |
| macOS (Intel) | `TonerScope_x.x.x_x64.dmg` |
| Linux (Debian/Ubuntu) | `tonerscope_x.x.x_amd64.deb` |
| Linux (AppImage) | `tonerscope_x.x.x_amd64.AppImage` |

---

## 🛠 Сборка из исходников

### Требования

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) stable
- Linux: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `libssl-dev`

```bash
# Клонировать репозиторий
git clone https://github.com/YOUR_USERNAME/tonerscope.git
cd tonerscope

# Установить зависимости
npm install

# Запустить в режиме разработки
npm run tauri dev

# Собрать релизную версию
npm run tauri build
```

Артефакты сборки будут в `src-tauri/target/release/bundle/`.

---

## ⚙️ Настройка

1. Запустите TonerScope
2. Перейдите в **Настройки**:
   - Укажите SNMP Community String (по умолчанию `public`)
   - Добавьте подсети для сканирования
   - Настройте пороги уведомлений
3. На странице **Сканирование** запустите поиск принтеров
4. Найденные принтеры добавляются в список одним кликом

### Требования к принтерам

Принтеры должны:
- Иметь сетевой интерфейс (LAN/Wi-Fi)
- Поддерживать SNMP v1 или v2c
- Быть доступны по UDP/161 из компьютера с TonerScope
- Community string должен совпадать (по умолчанию `public`)

---

## 📡 Поддерживаемые принтеры

| Бренд | SNMP | Расходники | Счётчик страниц |
|---|---|---|---|
| **Pantum** | ✅ | ✅ Тонер, Барабан | ✅ |
| **Kyocera** | ✅ | ✅ Тонер, Барабан, Термоузел | ✅ |
| **HP** | ✅ | ✅ Тонер | ✅ |
| **Canon** | ✅ | ✅ Тонер | ✅ |
| Другие (стандарт Printer MIB) | ✅ | ✅ | ✅ |

---

## 🏗 Архитектура

```
tonerscope/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # Компоненты UI
│   │   │   ├── ui/         # Базовые: Button, Card, Modal, ...
│   │   │   ├── printer/    # PrinterCard, TonerGauge, ...
│   │   │   └── layout/     # Sidebar, Header, PageWrapper
│   │   ├── stores/         # Svelte stores (состояние)
│   │   ├── styles/         # SCSS: переменные, миксины, анимации
│   │   ├── types/          # TypeScript типы
│   │   └── utils/          # Форматтеры, цвета
│   └── routes/             # Страницы приложения
└── src-tauri/              # Rust backend
    └── src/
        ├── snmp/           # SNMP клиент + OID маппинг
        ├── scanner/        # Сканирование подсетей
        ├── db/             # SQLite хранилище
        ├── commands/       # Tauri IPC команды
        └── scheduler/      # Фоновый опрос принтеров
```

---

## 📋 Релиз новой версии

```bash
# Обновить версию в package.json и src-tauri/Cargo.toml
# Затем:
git tag v0.2.0
git push origin v0.2.0
# GitHub Actions автоматически соберёт и опубликует релиз
```

---

## 📄 Лицензия

MIT © TonerScope Contributors
