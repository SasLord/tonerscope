<!-- src/lib/components/printer/PrinterGrid.svelte -->

<script lang="ts">
  import type { PrinterInfo } from '$lib/types/printer';
  import { onMount } from 'svelte';
  import PrinterCard from './PrinterCard.svelte';
  import { selectedPrinterId } from '$lib/stores/printers';
  import { api } from '$lib/api/tauri';

  export let printers: PrinterInfo[] = [];
  export let loading = false;

  // Определяется один раз при монтировании компонента
  let isWindows = false;

  onMount(() => {
    api.getSpoolerStatus()
      .then((status) => { isWindows = status !== 'unavailable'; })
      .catch(() => { isWindows = false; });
  });
</script>

{#if loading}
  <div class="grid">
    {#each Array(6) as _}
      <div class="skeleton-card">
        <div class="skeleton-card__header">
          <div class="sk sk--avatar"></div>
          <div class="sk-lines">
            <div class="sk sk--title"></div>
            <div class="sk sk--sub"></div>
          </div>
        </div>
        <div class="sk sk--bar"></div>
        <div class="sk sk--bar sk--bar-short"></div>
        <div class="sk sk--footer"></div>
      </div>
    {/each}
  </div>
{:else if printers.length === 0}
  <div class="empty">
    <div class="empty__icon">🖨</div>
    <p class="empty__title">Принтеры не найдены</p>
    <p class="empty__hint">Добавьте принтер вручную или выполните сканирование сети</p>
  </div>
{:else}
  <div class="grid">
    {#each printers as printer, i (printer.id)}
      <div class="animate-fade-in-up stagger-{Math.min(i + 1, 12)}">
        <PrinterCard
          {printer}
          {isWindows}
          selected={$selectedPrinterId === printer.id}
          on:select={(e) => selectedPrinterId.set(e.detail)}
        />
      </div>
    {/each}
  </div>
{/if}

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .grid {
    @include m.grid-auto-fill(260px);
    gap: v.$space-4;
    align-items: start;
  }

  // Skeleton cards
  .skeleton-card {
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: v.$radius-lg;
    padding: v.$space-4;
    display: flex;
    flex-direction: column;
    gap: v.$space-3;

    &__header {
      @include m.flex-start;
      gap: v.$space-3;
    }
  }

  .sk-lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
  }

  .sk {
    background: linear-gradient(90deg, var(--skeleton-base) 0%, var(--skeleton-shine) 50%, var(--skeleton-base) 100%);
    background-size: 400px 100%;
    animation: shimmer 1.6s ease-in-out infinite;
    border-radius: v.$radius-sm;

    &--avatar { width: 36px; height: 36px; border-radius: v.$radius-md; flex-shrink: 0; }
    &--title  { height: 13px; width: 70%; }
    &--sub    { height: 10px; width: 45%; }
    &--bar    { height: 5px; border-radius: v.$radius-full; }
    &--bar-short { width: 60%; }
    &--footer { height: 1px; background: var(--border); }
  }

  @keyframes shimmer {
    0%   { background-position: -400px 0; }
    100% { background-position: 400px 0; }
  }

  // Empty state
  .empty {
    grid-column: 1 / -1;
    @include m.flex-center;
    flex-direction: column;
    gap: v.$space-3;
    padding: v.$space-16;
    text-align: center;

    &__icon {
      font-size: 48px;
      filter: grayscale(1) opacity(0.3);
    }

    &__title {
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-semibold;
      color: var(--text-secondary);
    }

    &__hint {
      font-size: v.$font-size-sm;
      color: var(--text-tertiary);
    }
  }
</style>
