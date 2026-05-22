<!-- src/routes/+layout.svelte -->

<script lang="ts">
  import '../app.scss';
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { theme } from '$lib/stores/settings';
  import { notifications } from '$lib/stores/notifications';
  import { initPrinters, destroyPrinters } from '$lib/stores/printers';
  import { api, type UnlistenFn } from '$lib/api/tauri';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';

  let sidebarCollapsed = false;
  // Храним unlisten снаружи onMount — чтобы cleanup не зависел от async контекста
  let unlistenAlert: UnlistenFn | null = null;
  let initialized = false;

  // cleanup вынесен в синхронную функцию — вызывается из $effect cleanup
  function cleanup() {
    if (!initialized) return;
    destroyPrinters();
    unlistenAlert?.();
    unlistenAlert = null;
  }

  onMount(() => {
    theme.init();

    if (browser) {
      const saved = localStorage.getItem('sidebar-collapsed');
      if (saved !== null) sidebarCollapsed = saved === 'true';
    }

    // Запускаем async инициализацию — НЕ делаем onMount async,
    // чтобы не потерять контекст компонента в Svelte 5
    initPrinters().then(() => {
      initialized = true;
    });

    api.onPrinterAlert(({ ip, supply, percent }) => {
      notifications.warning(`Низкий тонер: ${supply}`, `${ip} — ${percent}%`);
    }).then(fn => {
      unlistenAlert = fn;
    });

    // onMount возвращает синхронную функцию cleanup — это корректно в Svelte 4 и 5
    return cleanup;
  });

  $: if (browser && typeof localStorage !== 'undefined') {
    localStorage.setItem('sidebar-collapsed', String(sidebarCollapsed));
  }
</script>

<div class="app-shell">
  <Sidebar bind:collapsed={sidebarCollapsed} />

  <div class="app-shell__content">
    <slot />
  </div>
</div>

<Toast />

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .app-shell {
    display: flex;
    min-height: 100vh;
    height: 100vh;
    overflow: hidden;
    background: var(--bg);

    &__content {
      flex: 1;
      min-width: 0;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
  }
</style>
