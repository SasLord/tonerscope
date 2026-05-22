// svelte.config.js

import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      pages:    'build',
      assets:   'build',
      fallback: 'index.html',
    }),
    alias: {
      $lib:                 './src/lib',
      '$app/environment':   './.svelte-kit/ambient.d.ts',
      '$app/stores':        './.svelte-kit/ambient.d.ts',
      '$app/navigation':    './.svelte-kit/ambient.d.ts',
      '$app/paths':         './.svelte-kit/ambient.d.ts',
    },
  },
};

export default config;
