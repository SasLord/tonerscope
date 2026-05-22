<!-- src/routes/+page.svelte -->

<script lang="ts">
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import PrinterGrid from '$lib/components/printer/PrinterGrid.svelte';
  import PrinterDetail from '$lib/components/printer/PrinterDetail.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { printers, printerStats, selectedPrinterId } from '$lib/stores/printers';
  import { formatPageCount } from '$lib/utils/formatters';

  const statCards = [
    { key: 'total',    label: 'Всего принтеров', icon: '🖨',  color: 'accent'   },
    { key: 'online',   label: 'В сети',           icon: '◉',   color: 'success'  },
    { key: 'errors',   label: 'Ошибки',           icon: '⚠',   color: 'error'    },
    { key: 'lowToner', label: 'Мало тонера',      icon: '↓',   color: 'warning'  },
  ] as const;

  $: sortedPrinters = [...$printers].sort((a, b) => {
    const aCrit = a.supplies.some(s => s.isCritical) ? 0 : a.supplies.some(s => s.isLow) ? 1 : 2;
    const bCrit = b.supplies.some(s => s.isCritical) ? 0 : b.supplies.some(s => s.isLow) ? 1 : 2;
    if (aCrit !== bCrit) return aCrit - bCrit;
    const aMin = Math.min(...a.supplies.map(s => s.percent), 100);
    const bMin = Math.min(...b.supplies.map(s => s.percent), 100);
    return aMin - bMin;
  });

  $: selectedPrinter = $selectedPrinterId
    ? $printers.find(p => p.id === $selectedPrinterId) ?? null
    : null;
</script>

<Header>
  <svelte:fragment slot="actions">
    <Button variant="primary" size="sm" on:click={() => window.location.href = '/scan'}>
      + Сканировать
    </Button>
  </svelte:fragment>
</Header>

<div class="dashboard">
  <div class="dashboard__main">
    <PageWrapper>
      <!-- Stat cards -->
      <div class="stat-grid">
        {#each statCards as card, i}
          {@const val = $printerStats[card.key]}
          <div
            class="stat-card stat-card--{card.color}"
            class:stat-card--alert={val > 0 && (card.key === 'errors' || card.key === 'lowToner')}
            style="animation-delay: {i * 60}ms"
          >
            <div class="stat-card__icon">{card.icon}</div>
            <div class="stat-card__body">
              <span class="stat-card__value">{val}</span>
              <span class="stat-card__label">{card.label}</span>
            </div>
            {#if card.key === 'errors' && val > 0}
              <div class="stat-card__pulse"></div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Section header -->
      <div class="section-header">
        <h2 class="section-header__title">Принтеры</h2>
        <span class="section-header__count">{$printers.length}</span>
        <div class="section-header__divider"></div>
        {#if $printerStats.critical > 0}
          <span class="section-header__alert">
            {$printerStats.critical} требуют внимания
          </span>
        {/if}
      </div>

      <PrinterGrid printers={sortedPrinters} />
    </PageWrapper>
  </div>

  <!-- Detail panel -->
  {#if selectedPrinter}
    <PrinterDetail
      printer={selectedPrinter}
      on:close={() => selectedPrinterId.set(null)}
    />
  {/if}
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .dashboard {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;

    &__main {
      flex: 1;
      min-width: 0;
      overflow-y: auto;
      @include m.custom-scrollbar;
    }
  }

  // ── Stat cards ──────────────────────────────────────────────────────────────
  .stat-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: v.$space-4;
    margin-bottom: v.$space-8;
  }

  .stat-card {
    @include m.card-base;
    box-shadow: var(--shadow-card);
    padding: v.$space-4 v.$space-5;
    @include m.flex-start;
    gap: v.$space-4;
    position: relative;
    overflow: hidden;
    animation: fadeInUp 350ms ease both;

    &__icon {
      font-size: 22px;
      flex-shrink: 0;
      line-height: 1;
    }

    &__body {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }

    &__value {
      font-family: v.$font-display;
      font-size: v.$font-size-2xl;
      font-weight: v.$font-weight-black;
      line-height: 1;
      letter-spacing: v.$letter-spacing-tight;
      color: var(--text-primary);
    }

    &__label {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      font-weight: v.$font-weight-medium;
    }

    &__pulse {
      position: absolute;
      top: v.$space-3;
      right: v.$space-3;
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--status-error);
      &::before {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        background: inherit;
        animation: ping 1.5s ease-in-out infinite;
      }
    }

    &--accent  .stat-card__value { color: var(--accent); }
    &--success .stat-card__value { color: var(--status-online); }
    &--error   .stat-card__value { color: var(--status-error); }
    &--warning .stat-card__value { color: var(--status-warning); }

    &--alert {
      &.stat-card--error {
        border-color: rgba(239,68,68,0.25);
        background: linear-gradient(135deg, var(--surface-1) 0%, rgba(239,68,68,0.04) 100%);
      }
      &.stat-card--warning {
        border-color: rgba(245,158,11,0.2);
        background: linear-gradient(135deg, var(--surface-1) 0%, rgba(245,158,11,0.04) 100%);
      }
    }

    @keyframes ping {
      0%        { transform: scale(1); opacity: 0.7; }
      70%, 100% { transform: scale(2.5); opacity: 0; }
    }

    @keyframes fadeInUp {
      from { opacity: 0; transform: translateY(10px); }
      to   { opacity: 1; transform: translateY(0); }
    }
  }

  // ── Section header ───────────────────────────────────────────────────────────
  .section-header {
    @include m.flex-start;
    gap: v.$space-3;
    margin-bottom: v.$space-4;

    &__title {
      font-size: v.$font-size-md;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }

    &__count {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      background: var(--surface-2);
      border: 1px solid var(--border);
      border-radius: v.$radius-full;
      padding: 1px 8px;
    }

    &__divider {
      flex: 1;
      height: 1px;
      background: var(--border);
    }

    &__alert {
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-semibold;
      color: var(--status-warning);
    }
  }
</style>
