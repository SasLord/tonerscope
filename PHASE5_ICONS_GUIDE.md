# Фаза 5 — Иконки и брендинг: инструкция

## Файлы из этой фазы

| Файл | Куда класть | Назначение |
|---|---|---|
| `favicon.svg` | `static/favicon.svg` | Иконка для браузерной вкладки (адаптивная: светлая/тёмная тема) |
| `tonerscope-1024.svg` | `src-tauri/icons/tonerscope-1024.svg` | Мастер-иконка для генерации всех форматов |
| `tray-icon.svg` | `src-tauri/icons/tray-icon.svg` | Монохромная tray-иконка 22×22 |

---

## Шаг 1 — Конвертировать SVG → PNG (1024×1024)

`npx tauri icon` принимает **PNG**, не SVG. Конвертируй мастер-иконку любым удобным способом:

### Вариант A — Inkscape (рекомендуется, точная цветопередача)
```bash
inkscape src-tauri/icons/tonerscope-1024.svg \
  --export-type=png \
  --export-filename=src-tauri/icons/tonerscope-1024.png \
  --export-width=1024 \
  --export-height=1024
```

### Вариант B — rsvg-convert (Linux)
```bash
rsvg-convert -w 1024 -h 1024 \
  src-tauri/icons/tonerscope-1024.svg \
  -o src-tauri/icons/tonerscope-1024.png
```

### Вариант C — sharp (Node.js, кроссплатформенно)
```bash
npx sharp-cli --input src-tauri/icons/tonerscope-1024.svg \
  --output src-tauri/icons/tonerscope-1024.png \
  resize 1024 1024
```

### Вариант D — Python + cairosvg
```bash
pip install cairosvg
python3 -c "
import cairosvg
cairosvg.svg2png(
  url='src-tauri/icons/tonerscope-1024.svg',
  write_to='src-tauri/icons/tonerscope-1024.png',
  output_width=1024, output_height=1024
)"
```

---

## Шаг 2 — Сгенерировать все иконки Tauri

```bash
npx tauri icon src-tauri/icons/tonerscope-1024.png
```

Команда автоматически создаёт в `src-tauri/icons/`:
- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

Эти файлы уже прописаны в `tauri.conf.json` → `bundle.icon` — ничего менять не нужно.

---

## Шаг 3 — Tray-иконка

Конвертируй `tray-icon.svg` в PNG:

```bash
# Inkscape
inkscape src-tauri/icons/tray-icon.svg \
  --export-type=png \
  --export-filename=src-tauri/icons/tray-icon.png \
  --export-width=22 \
  --export-height=22

# rsvg-convert
rsvg-convert -w 22 -h 22 \
  src-tauri/icons/tray-icon.svg \
  -o src-tauri/icons/tray-icon.png
```

### Подключение в Rust (если нужен трей)

В `src-tauri/src/lib.rs` добавить при инициализации:

```rust
use tauri::tray::TrayIconBuilder;

// внутри .setup(|app| { ... })
TrayIconBuilder::new()
  .icon(app.default_window_icon().unwrap().clone())
  // или явно:
  // .icon(tauri::image::Image::from_path("icons/tray-icon.png").unwrap())
  .build(app)?;
```

---

## Шаг 4 — favicon.png для static/

```bash
# Inkscape
inkscape static/favicon.svg \
  --export-type=png \
  --export-filename=static/favicon.png \
  --export-width=32 \
  --export-height=32
```

---

## Примечания

- `favicon.svg` адаптируется к теме ОС через `@media (prefers-color-scheme)` — работает в браузерном окне Tauri независимо от темы приложения.
- Tray SVG по умолчанию белый (для тёмной системной панели). Для светлой панели замени `#ffffff` на `#000000`.
- Tauri v2 поддерживает `TrayIcon` из `tauri::tray` — плагин `tauri-plugin-tray` не нужен.
