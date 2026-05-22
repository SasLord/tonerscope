<!-- src/lib/components/layout/Header.svelte -->

<script lang="ts">
  import { page } from '$app/stores';
  import { printerStats } from '$lib/stores/printers';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';

  export let title = '';
  export let subtitle = '';

  export let searchable = false;
  export let searchValue = '';

  const pageTitles: Record<string, { title: string; subtitle: string }> = {
    '/':          { title: 'Обзор',         subtitle: 'Общее состояние парка принтеров' },
    '/printers':  { title: 'Принтеры',       subtitle: 'Все принтеры в сети' },
    '/scan':      { title: 'Сканирование',   subtitle: 'Поиск принтеров в локальной сети' },
    '/history':   { title: 'История',        subtitle: 'Динамика уровней расходников' },
    '/alerts':    { title: 'Уведомления',    subtitle: 'Правила оповещений' },
    '/settings':  { title: 'Настройки',      subtitle: 'Конфигурация приложения' },
  };

  $: currentPath = $page.url.pathname;
  $: pageInfo = pageTitles[currentPath] ?? { title, subtitle };
</script>

<header class="header" data-tauri-drag-region>
  <div class="header__left">
    <div class="header__titles">
      <h1 class="header__title">{pageInfo.title}</h1>
      {#if pageInfo.subtitle}
        <p class="header__subtitle">{pageInfo.subtitle}</p>
      {/if}
    </div>
  </div>

  <div class="header__right">
    {#if searchable}
      <div class="header__search">
        <Input
          type="search"
          placeholder="Поиск принтера..."
          bind:value={searchValue}
        >
          <svelte:fragment slot="prefix">
            <span style="font-size: 13px;">⌕</span>
          </svelte:fragment>
        </Input>
      </div>
    {/if}

    <!-- Quick status chips -->
    <div class="header__stats">
      {#if $printerStats.errors > 0}
        <a href="/printers" class="header__stat-chip header__stat-chip--error">
          ⚠ {$printerStats.errors} ошибок
        </a>
      {/if}
      {#if $printerStats.critical > 0}
        <a href="/alerts" class="header__stat-chip header__stat-chip--crit">
          ! {$printerStats.critical} критичн.
        </a>
      {/if}
    </div>

    <slot name="actions" />
  </div>
</header>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .header {
    height: v.$header-height;
    @include m.flex-between;
    gap: v.$space-4;
    padding: 0 v.$space-6;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    background: var(--bg);

    &__left { @include m.flex-start; gap: v.$space-4; min-width: 0; }
    &__right { @include m.flex-start; gap: v.$space-3; flex-shrink: 0; }

    &__titles {
      display: flex;
      align-items: baseline;
      gap: v.$space-3;
    }

    &__title {
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
      white-space: nowrap;
    }

    &__subtitle {
      font-size: v.$font-size-sm;
      color: var(--text-tertiary);
      white-space: nowrap;
      @include m.respond-below('lg') { display: none; }
    }

    &__search { width: 220px; }

    &__stats {
      @include m.flex-start;
      gap: v.$space-2;
    }

    &__stat-chip {
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-bold;
      padding: 3px 9px;
      border-radius: v.$radius-full;
      text-decoration: none;
      transition: opacity v.$transition-fast;
      &:hover { opacity: 0.8; }

      &--error {
        background: rgba(239,68,68,0.12);
        color: var(--status-error);
        border: 1px solid rgba(239,68,68,0.25);
      }
      &--crit {
        background: rgba(245,158,11,0.12);
        color: var(--status-warning);
        border: 1px solid rgba(245,158,11,0.25);
      }
    }
  }
</style>
