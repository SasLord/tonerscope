<!-- src/lib/components/layout/Sidebar.svelte -->

<script lang="ts">
  import { page } from '$app/stores';
  import { theme } from '$lib/stores/settings';
  import { printerStats } from '$lib/stores/printers';
  import Tooltip from '$lib/components/ui/Tooltip.svelte';

  export let collapsed = false;

  interface NavItem {
    href:    string;
    label:   string;
    icon:    string;
    badge?:  () => number | null;
  }

  const nav: NavItem[] = [
    { href: '/',          label: 'Обзор',         icon: '⬡' },
    { href: '/printers',  label: 'Принтеры',       icon: '🖨', badge: () => $printerStats.total || null },
    { href: '/scan',      label: 'Сканирование',   icon: '◎' },
    { href: '/history',   label: 'История',        icon: '◷' },
    { href: '/alerts',    label: 'Уведомления',    icon: '◬', badge: () => $printerStats.critical || null },
    { href: '/settings',  label: 'Настройки',      icon: '⚙' },
  ];

  $: currentPath = $page.url.pathname;
  $: isActive = (href: string) =>
    href === '/' ? currentPath === '/' : currentPath.startsWith(href);
</script>

<aside class="sidebar" class:sidebar--collapsed={collapsed}>
  <!-- Logo -->
  <div class="sidebar__logo" data-tauri-drag-region>
    <div class="sidebar__logo-mark">
      <span class="sidebar__logo-t">T</span>
    </div>
    {#if !collapsed}
      <div class="sidebar__brand">
        <span class="sidebar__brand-name">TonerScope</span>
        <span class="sidebar__brand-ver">v0.1.0</span>
      </div>
    {/if}
  </div>

  <!-- Alert summary -->
  {#if !collapsed && ($printerStats.critical > 0 || $printerStats.errors > 0)}
    <div class="sidebar__alert-bar">
      <span class="sidebar__alert-dot"></span>
      {#if $printerStats.critical > 0}
        {$printerStats.critical} критичн.
      {:else}
        {$printerStats.errors} ошибок
      {/if}
    </div>
  {/if}

  <!-- Navigation -->
  <nav class="sidebar__nav" aria-label="Навигация">
    <ul class="sidebar__nav-list">
      {#each nav as item}
        {@const active = isActive(item.href)}
        {@const badgeVal = item.badge?.() ?? null}
        <li>
          {#if collapsed}
            <Tooltip text={item.label} position="right">
              <a href={item.href} class="sidebar__link" class:sidebar__link--active={active} aria-current={active ? 'page' : undefined}>
                <span class="sidebar__link-icon" aria-hidden="true">{item.icon}</span>
              </a>
            </Tooltip>
          {:else}
            <a href={item.href} class="sidebar__link" class:sidebar__link--active={active} aria-current={active ? 'page' : undefined}>
              <span class="sidebar__link-icon" aria-hidden="true">{item.icon}</span>
              <span class="sidebar__link-label">{item.label}</span>
              {#if badgeVal}
                <span class="sidebar__link-badge" class:sidebar__link-badge--alert={item.href === '/alerts'}>
                  {badgeVal}
                </span>
              {/if}
            </a>
          {/if}
        </li>
      {/each}
    </ul>
  </nav>

  <!-- Bottom: theme toggle + collapse -->
  <div class="sidebar__bottom">
    <Tooltip text={$theme === 'dark' ? 'Светлая тема' : 'Тёмная тема'} position={collapsed ? 'right' : 'top'}>
      <button class="sidebar__icon-btn" on:click={() => theme.toggle()} aria-label="Переключить тему">
        {$theme === 'dark' ? '☀' : '☾'}
      </button>
    </Tooltip>

    <Tooltip text={collapsed ? 'Развернуть' : 'Свернуть'} position={collapsed ? 'right' : 'top'}>
      <button
        class="sidebar__icon-btn sidebar__collapse-btn"
        on:click={() => collapsed = !collapsed}
        aria-label={collapsed ? 'Развернуть меню' : 'Свернуть меню'}
      >
        {collapsed ? '›' : '‹'}
      </button>
    </Tooltip>
  </div>
</aside>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .sidebar {
    width: v.$sidebar-width;
    min-height: 100vh;
    background: var(--sidebar-bg);
    border-right: 1px solid var(--sidebar-border);
    display: flex;
    flex-direction: column;
    transition: width v.$transition-slow;
    flex-shrink: 0;
    overflow: hidden;
    position: relative;

    &--collapsed {
      width: v.$sidebar-collapsed-w;

      .sidebar__logo { justify-content: center; padding: v.$space-4 0; }
      .sidebar__nav-list { padding: v.$space-2 v.$space-3; }
      .sidebar__bottom   { justify-content: center; flex-direction: column; }
    }

    // ── Logo ──
    &__logo {
      @include m.flex-start;
      gap: v.$space-3;
      padding: v.$space-5 v.$space-4;
      border-bottom: 1px solid var(--sidebar-border);
      flex-shrink: 0;
    }

    &__logo-mark {
      width: 32px;
      height: 32px;
      border-radius: v.$radius-md;
      background: var(--accent);
      @include m.flex-center;
      flex-shrink: 0;
    }

    &__logo-t {
      font-family: v.$font-display;
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-black;
      color: var(--accent-fg);
      line-height: 1;
    }

    &__brand { display: flex; flex-direction: column; }
    &__brand-name {
      font-family: v.$font-display;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
      letter-spacing: -0.02em;
    }
    &__brand-ver {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    // ── Alert bar ──
    &__alert-bar {
      @include m.flex-start;
      gap: v.$space-2;
      margin: v.$space-3 v.$space-3 0;
      padding: v.$space-2 v.$space-3;
      background: rgba(239,68,68,0.08);
      border: 1px solid rgba(239,68,68,0.2);
      border-radius: v.$radius-md;
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-semibold;
      color: var(--status-error);
    }

    &__alert-dot {
      width: 6px; height: 6px;
      border-radius: 50%;
      background: var(--status-error);
      flex-shrink: 0;
      animation: pulse 1.5s ease-in-out infinite;
    }

    // ── Nav ──
    &__nav { flex: 1; overflow-y: auto; @include m.custom-scrollbar; }
    &__nav-list {
      display: flex;
      flex-direction: column;
      gap: 2px;
      padding: v.$space-3;
    }

    &__link {
      @include m.flex-start;
      gap: v.$space-3;
      padding: v.$space-2 v.$space-3;
      border-radius: v.$radius-md;
      color: var(--text-secondary);
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-medium;
      transition: background v.$transition-fast, color v.$transition-fast;
      text-decoration: none;
      white-space: nowrap;
      @include m.focus-ring;

      &:hover {
        background: var(--nav-hover-bg);
        color: var(--text-primary);
      }

      &--active {
        background: var(--nav-active-bg);
        color: var(--nav-active-text);
        font-weight: v.$font-weight-semibold;
      }
    }

    &__link-icon {
      font-size: 16px;
      flex-shrink: 0;
      width: 20px;
      text-align: center;
      line-height: 1;
    }

    &__link-label { flex: 1; }

    &__link-badge {
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-bold;
      background: var(--surface-3);
      color: var(--text-secondary);
      padding: 1px 6px;
      border-radius: v.$radius-full;
      min-width: 18px;
      text-align: center;

      &--alert {
        background: rgba(239,68,68,0.15);
        color: var(--status-error);
      }
    }

    // ── Bottom ──
    &__bottom {
      @include m.flex-between;
      padding: v.$space-3 v.$space-4;
      border-top: 1px solid var(--sidebar-border);
      flex-shrink: 0;
      gap: v.$space-2;
    }

    &__icon-btn {
      width: 30px;
      height: 30px;
      @include m.flex-center;
      border-radius: v.$radius-md;
      color: var(--text-tertiary);
      font-size: v.$font-size-md;
      transition: background v.$transition-fast, color v.$transition-fast;
      @include m.focus-ring;

      &:hover {
        background: var(--nav-hover-bg);
        color: var(--text-primary);
      }
    }

    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50%       { opacity: 0.4; }
    }
  }
</style>
