<!-- src/routes/alerts/+page.svelte -->

<script lang="ts">
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { printers } from '$lib/stores/printers';
  import { supplyLabel, statusLabel } from '$lib/utils/formatters';
  import { tonerColor } from '$lib/utils/colors';

  // Critical and low printers
  $: criticalItems = $printers.flatMap(p =>
    p.supplies
      .filter(s => s.isCritical)
      .map(s => ({ printer: p, supply: s }))
  );

  $: lowItems = $printers.flatMap(p =>
    p.supplies
      .filter(s => s.isLow && !s.isCritical)
      .map(s => ({ printer: p, supply: s }))
  );

  $: offlineItems = $printers.filter(p => p.status === 'offline' || p.status === 'error');
</script>

<Header />

<PageWrapper>
  <div class="alerts-layout">

    <!-- Critical section -->
    {#if criticalItems.length > 0}
      <section class="alert-section">
        <div class="alert-section__header alert-section__header--critical">
          <span class="alert-section__icon">⚠</span>
          <h2 class="alert-section__title">Критические ({criticalItems.length})</h2>
          <span class="alert-section__desc">Требуется немедленная замена расходника</span>
        </div>
        <div class="alert-cards">
          {#each criticalItems as { printer, supply }}
            <div class="alert-card alert-card--critical animate-fade-in-up">
              <div class="alert-card__left">
                <div class="alert-card__icon">!</div>
                <div class="alert-card__info">
                  <span class="alert-card__printer">{printer.name}</span>
                  <span class="alert-card__supply">{supplyLabel(supply.type)}</span>
                  <span class="alert-card__ip">{printer.ip}</span>
                </div>
              </div>
              <div class="alert-card__right">
                <span class="alert-card__pct" style="color: {tonerColor(supply.percent)}">
                  {supply.percent}%
                </span>
                <div class="alert-card__bar">
                  <div class="alert-card__bar-fill" style="width: {supply.percent}%; background: {tonerColor(supply.percent)};"></div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Low toner section -->
    {#if lowItems.length > 0}
      <section class="alert-section">
        <div class="alert-section__header alert-section__header--warning">
          <span class="alert-section__icon">↓</span>
          <h2 class="alert-section__title">Заканчивается ({lowItems.length})</h2>
          <span class="alert-section__desc">Уровень расходника ниже порогового значения</span>
        </div>
        <div class="alert-cards">
          {#each lowItems as { printer, supply }}
            <div class="alert-card alert-card--warning animate-fade-in-up">
              <div class="alert-card__left">
                <div class="alert-card__icon alert-card__icon--warning">↓</div>
                <div class="alert-card__info">
                  <span class="alert-card__printer">{printer.name}</span>
                  <span class="alert-card__supply">{supplyLabel(supply.type)}</span>
                  <span class="alert-card__ip">{printer.ip}</span>
                </div>
              </div>
              <div class="alert-card__right">
                <span class="alert-card__pct" style="color: {tonerColor(supply.percent)}">
                  {supply.percent}%
                </span>
                <div class="alert-card__bar">
                  <div class="alert-card__bar-fill" style="width: {supply.percent}%; background: {tonerColor(supply.percent)};"></div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Offline/Error section -->
    {#if offlineItems.length > 0}
      <section class="alert-section">
        <div class="alert-section__header alert-section__header--offline">
          <span class="alert-section__icon">◌</span>
          <h2 class="alert-section__title">Недоступны ({offlineItems.length})</h2>
          <span class="alert-section__desc">Принтеры не отвечают или в состоянии ошибки</span>
        </div>
        <div class="alert-cards">
          {#each offlineItems as printer}
            <div class="alert-card alert-card--offline animate-fade-in-up">
              <div class="alert-card__left">
                <div class="alert-card__icon alert-card__icon--offline">◌</div>
                <div class="alert-card__info">
                  <span class="alert-card__printer">{printer.name}</span>
                  <span class="alert-card__ip">{printer.ip}</span>
                </div>
              </div>
              <div class="alert-card__right">
                <Badge variant={printer.status === 'error' ? 'error' : 'neutral'}>
                  {statusLabel(printer.status)}
                </Badge>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- All clear -->
    {#if criticalItems.length === 0 && lowItems.length === 0 && offlineItems.length === 0}
      <div class="all-clear">
        <div class="all-clear__icon">✓</div>
        <h2 class="all-clear__title">Всё в порядке</h2>
        <p class="all-clear__desc">Все принтеры работают нормально, расходники в норме</p>
      </div>
    {/if}

  </div>
</PageWrapper>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .alerts-layout {
    display: flex;
    flex-direction: column;
    gap: v.$space-6;
  }

  .alert-section {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;

    &__header {
      @include m.flex-start;
      gap: v.$space-3;
      padding: v.$space-3 v.$space-4;
      border-radius: v.$radius-md;
      border: 1px solid;

      &--critical {
        background: rgba(239,68,68,0.06);
        border-color: rgba(239,68,68,0.2);
        .alert-section__icon { color: var(--status-error); }
        .alert-section__title { color: var(--status-error); }
      }
      &--warning {
        background: rgba(245,158,11,0.06);
        border-color: rgba(245,158,11,0.2);
        .alert-section__icon { color: var(--status-warning); }
        .alert-section__title { color: var(--status-warning); }
      }
      &--offline {
        background: rgba(113,113,122,0.06);
        border-color: var(--border);
        .alert-section__icon { color: var(--text-tertiary); }
        .alert-section__title { color: var(--text-secondary); }
      }
    }

    &__icon {
      font-size: v.$font-size-lg;
      flex-shrink: 0;
    }

    &__title {
      font-size: v.$font-size-base;
      font-weight: v.$font-weight-bold;
    }

    &__desc {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
    }
  }

  .alert-cards {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
  }

  .alert-card {
    @include m.flex-between;
    gap: v.$space-4;
    padding: v.$space-3 v.$space-4;
    border-radius: v.$radius-md;
    border: 1px solid;
    transition: border-color v.$transition-fast;

    &--critical {
      background: rgba(239,68,68,0.04);
      border-color: rgba(239,68,68,0.15);
      &:hover { border-color: rgba(239,68,68,0.3); }
      .alert-card__icon {
        background: rgba(239,68,68,0.15);
        color: var(--status-error);
      }
    }
    &--warning {
      background: rgba(245,158,11,0.04);
      border-color: rgba(245,158,11,0.15);
      &:hover { border-color: rgba(245,158,11,0.3); }
      .alert-card__icon--warning {
        background: rgba(245,158,11,0.15);
        color: var(--status-warning);
      }
    }
    &--offline {
      background: var(--surface-1);
      border-color: var(--border);
      .alert-card__icon--offline {
        background: var(--surface-2);
        color: var(--text-tertiary);
      }
    }

    &__left {
      @include m.flex-start;
      gap: v.$space-3;
      min-width: 0;
    }

    &__icon {
      width: 32px;
      height: 32px;
      border-radius: v.$radius-md;
      @include m.flex-center;
      font-size: v.$font-size-base;
      font-weight: v.$font-weight-black;
      flex-shrink: 0;
    }

    &__info {
      display: flex;
      flex-direction: column;
      gap: 1px;
      min-width: 0;
    }

    &__printer {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
      @include m.truncate;
    }

    &__supply {
      font-size: v.$font-size-xs;
      color: var(--text-secondary);
    }

    &__ip {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__right {
      @include m.flex-start;
      gap: v.$space-3;
      flex-shrink: 0;
    }

    &__pct {
      font-family: v.$font-mono;
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-black;
      min-width: 44px;
      text-align: right;
    }

    &__bar {
      width: 80px;
      height: 4px;
      background: var(--gauge-track);
      border-radius: v.$radius-full;
      overflow: hidden;

      &-fill {
        height: 100%;
        border-radius: v.$radius-full;
        transition: width 0.4s ease;
      }
    }
  }

  // All clear
  .all-clear {
    @include m.flex-center;
    flex-direction: column;
    gap: v.$space-4;
    padding: v.$space-16;
    text-align: center;

    &__icon {
      width: 64px;
      height: 64px;
      border-radius: 50%;
      background: rgba(34,197,94,0.12);
      border: 2px solid rgba(34,197,94,0.3);
      @include m.flex-center;
      font-size: 28px;
      color: var(--status-online);
    }

    &__title {
      font-size: v.$font-size-xl;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }

    &__desc {
      font-size: v.$font-size-sm;
      color: var(--text-tertiary);
    }
  }
</style>
