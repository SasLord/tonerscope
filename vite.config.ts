// vite.config.ts

import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],

  // Tauri: prevent Vite from obscuring Rust errors
  clearScreen: false,

  server: {
    port:         1420,
    strictPort:   true,
    watch: {
      // Tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },

  // Tauri expects a fixed port, fail if that port is not available
  envPrefix: ['VITE_', 'TAURI_'],
});
