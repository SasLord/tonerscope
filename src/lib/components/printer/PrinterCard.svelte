<!-- src/lib/components/printer/PrinterCard.svelte -->

<script lang="ts">
  import type { PrinterInfo } from '$lib/types/printer';
  import { createEventDispatcher } from 'svelte';
  import StatusBadge from './StatusBadge.svelte';
  import TonerGauge from './TonerGauge.svelte';
  import Tooltip from '$lib/components/ui/Tooltip.svelte';
  import { formatPageCount, formatRelativeTime, brandLabel } from '$lib/utils/formatters';

  export let printer: PrinterInfo;
  export let selected = false;

  const dispatch = createEventDispatcher<{ select: string }>();

  $: hasCritical = printer.supplies.some(s => s.isCritical);
  $: hasLow      = printer.supplies.some(s => s.isLow);
  $: alertLevel  = hasCritical ? 'critical' : hasLow ? 'low' : 'none';

  const brandIcons: Record<string, string> = {
    pantum:  'P',
    kyocera: 'K',
    hp:      'H',
    canon:   'C',
    other:   '?',
  };
</script>

<div
  class="printer-card"
  class:printer-card--selected={selected}
  class:printer-card--critical={alertLevel === 'critical'}
  class:printer-card--low={alertLevel === 'low'}
  class:printer-card--offline={printer.status === 'offline'}
  on:click={() => dispatch('select', printer.id)}
  on:keydown={(e) => e.key === 'Enter' && dispatch('select', printer.id)}
  tabindex="0"
  role="button"
  aria-pressed={selected}
>
  <!-- Top row: brand icon + name + status -->
  <div class="printer-card__header">
    <div class="printer-card__brand-badge" data-brand={printer.brand}>
      {brandIcons[printer.brand] ?? '?'}
    </div>
    <div class="printer-card__title-block">
      <h3 class="printer-card__name">{printer.name}</h3>
      <span class="printer-card__model">{printer.model}</span>
    </div>
    <StatusBadge status={printer.status} size="sm" />
  </div>

  <!-- Alert banner -->
  {#if alertLevel !== 'none'}
    <div class="printer-card__alert" class:printer-card__alert--critical={alertLevel === 'critical'}>
      {#if alertLevel === 'critical'}
        ⚠ Требуется замена расходника
      {:else}
        ↓ Расходник заканчивается
      {/if}
    </div>
  {/if}

  <!-- Supplies -->
  <div class="printer-card__supplies">
    {#each printer.supplies as supply (supply.type)}
      <TonerGauge {supply} compact />
    {/each}
  </div>

  <!-- Footer: IP + page count + last seen -->
  <div class="printer-card__footer">
    <div class="printer-card__meta">
      <Tooltip text="IP-адрес принтера">
        <span class="printer-card__ip">{printer.ip}</span>
      </Tooltip>
      {#if printer.location}
        <span class="printer-card__sep">·</span>
        <span class="printer-card__location">{printer.location}</span>
      {/if}
    </div>
    <div class="printer-card__stats">
      {#if printer.pageCount != null}
        <Tooltip text="Всего напечатано страниц">
          <span class="printer-card__pages">
            <span class="printer-card__pages-icon">□</span>
            {formatPageCount(printer.pageCount)}
          </span>
        </Tooltip>
      {/if}
      <Tooltip text="Последний ответ">
        <span class="printer-card__lastseen">{formatRelativeTime(printer.lastSeen)}</span>
      </Tooltip>
    </div>
  </div>
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .printer-card {
    @include m.card-base;
    box-shadow: var(--shadow-card);
    padding: v.$space-4;
    display: flex;
    flex-direction: column;
    gap: v.$space-3;
    cursor: pointer;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease,
      transform 0.15s ease;
    @include m.focus-ring;

    &:hover {
      border-color: var(--border-hover);
      box-shadow: v.$shadow-md;
      transform: translateY(-1px);
    }

    &:active { transform: translateY(0); }

    &--selected {
      border-color: var(--accent) !important;
      box-shadow: 0 0 0 3px var(--accent-muted), var(--shadow-card) !important;
    }

    &--critical {
      border-color: rgba(239,68,68,0.35) !important;
      &:hover { border-color: rgba(239,68,68,0.55) !important; }
    }

    &--low {
      border-color: rgba(245,158,11,0.3) !important;
    }

    &--offline {
      opacity: 0.65;
      filter: saturate(0.4);
    }

    // ── Header ──
    &__header {
      @include m.flex-start;
      gap: v.$space-3;
    }

    &__brand-badge {
      width: 36px;
      height: 36px;
      border-radius: v.$radius-md;
      @include m.flex-center;
      font-family: v.$font-display;
      font-size: v.$font-size-md;
      font-weight: v.$font-weight-black;
      flex-shrink: 0;

      &[data-brand='pantum']  { background: rgba(0,102,204,0.15); color: #4d9de0; }
      &[data-brand='kyocera'] { background: rgba(204,0,0,0.12);   color: #e05252; }
      &[data-brand='hp']      { background: rgba(0,150,214,0.15); color: #4ec0e8; }
      &[data-brand='canon']   { background: rgba(180,0,0,0.12);   color: #e07070; }
      &[data-brand='other']   { background: var(--surface-2);     color: var(--text-tertiary); }
    }

    &__title-block {
      flex: 1;
      min-width: 0;
    }

    &__name {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
      @include m.truncate;
      letter-spacing: -0.01em;
    }

    &__model {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
      display: block;
      margin-top: 1px;
    }

    // ── Alert ──
    &__alert {
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-semibold;
      letter-spacing: 0.04em;
      padding: v.$space-1 v.$space-3;
      border-radius: v.$radius-sm;
      background: rgba(245,158,11,0.10);
      color: var(--status-warning);
      border: 1px solid rgba(245,158,11,0.2);

      &--critical {
        background: rgba(239,68,68,0.10);
        color: var(--status-error);
        border-color: rgba(239,68,68,0.25);
      }
    }

    // ── Supplies ──
    &__supplies {
      display: flex;
      flex-direction: column;
      gap: v.$space-2;
    }

    // ── Footer ──
    &__footer {
      @include m.flex-between;
      gap: v.$space-2;
      padding-top: v.$space-2;
      border-top: 1px solid var(--border);
      margin-top: auto;
    }

    &__meta {
      @include m.flex-start;
      gap: v.$space-1;
      min-width: 0;
    }

    &__ip {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__sep {
      color: var(--text-tertiary);
      font-size: v.$font-size-xs;
    }

    &__location {
      font-size: 10px;
      color: var(--text-tertiary);
      @include m.truncate;
    }

    &__stats {
      @include m.flex-start;
      gap: v.$space-3;
      flex-shrink: 0;
    }

    &__pages {
      @include m.flex-start;
      gap: 3px;
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__pages-icon {
      font-size: 9px;
      opacity: 0.7;
    }

    &__lastseen {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }
  }
</style>
