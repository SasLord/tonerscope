// src/routes/+layout.ts
// Tauri использует статические файлы — SSR не нужен.
// prerender отключён: при prerender=true SvelteKit рендерит страницы
// в Node.js во время сборки, где window/__TAURI__ отсутствуют.
// adapter-static с fallback:'index.html' справляется без prerender.

export const ssr      = false;
export const prerender = false;
