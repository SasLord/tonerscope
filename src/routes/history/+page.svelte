<!-- src/routes/history/+page.svelte -->

<script lang="ts">
  import { onMount } from 'svelte';
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { printers } from '$lib/stores/printers';
  import { supplyLabel } from '$lib/utils/formatters';
  import { tonerColor, supplyTypeColor } from '$lib/utils/colors';
  import { api, type SnapshotRecord } from '$lib/api/tauri';
  import type { PrinterInfo, Supply, SupplyType } from '$lib/types/printer';

  // ─── Типы ────────────────────────────────────────────────────────────────────

  interface HistoryPoint {
    date: Date;
    pct:  number;
  }

  interface SupplyHistory {
    type:   SupplyType;
    name:   string;
    supply: Supply;
    points: HistoryPoint[];
  }

  interface PrinterHistory {
    printer:   PrinterInfo;
    histories: SupplyHistory[];
  }

  // ─── Состояние ───────────────────────────────────────────────────────────────

  let selectedIdx = 0;
  let historyData: (PrinterHistory | null)[] = [];
  let loading    = false;
  let loadError  = '';

  const cache = new Map<string, SnapshotRecord[]>();

  // ─── Загрузка ─────────────────────────────────────────────────────────────────

  async function loadHistory(printer: PrinterInfo): Promise<PrinterHistory> {
    let snapshots: SnapshotRecord[] = [];

    if (cache.has(printer.id)) {
      snapshots = cache.get(printer.id)!;
    } else {
      try {
        snapshots = await api.getSnapshots(printer.id, 90);
        cache.set(printer.id, snapshots);
      } catch (err) {
        console.error('[history] getSnapshots failed:', err);
      }
    }

    // Собираем набор типов расходников
    const supplyTypes = new Set<string>();
    printer.supplies.forEach(s => supplyTypes.add(s.type));
    snapshots.forEach(snap => {
      try {
        // suppliesJson — camelCase из serde(rename_all = "camelCase")
        const arr: Array<{ type: string }> = JSON.parse(snap.suppliesJson);
        arr.forEach(s => supplyTypes.add(s.type));
      } catch { /* битая запись */ }
    });

    const histories: SupplyHistory[] = Array.from(supplyTypes).map(sType => {
      const currentSupply = printer.supplies.find(s => s.type === sType);

      const points: HistoryPoint[] = snapshots
        .map(snap => {
          try {
            const arr: Array<{ type: string; percent: number }> =
              JSON.parse(snap.suppliesJson);
            const found = arr.find(s => s.type === sType);
            if (!found) return null;
            return { date: new Date(snap.timestamp), pct: found.percent };
          } catch {
            return null;
          }
        })
        .filter((p): p is HistoryPoint => p !== null)
        .sort((a, b) => a.date.getTime() - b.date.getTime());

      // Если история пустая — точка из текущего состояния
      if (points.length === 0 && currentSupply) {
        points.push({ date: new Date(), pct: currentSupply.percent });
      }

      const fallbackSupply: Supply = {
        type:       sType as SupplyType,
        name:       supplyLabel(sType as SupplyType),
        level:      0,
        maxLevel:   100,
        percent:    points.at(-1)?.pct ?? 0,
        isLow:      false,
        isCritical: false,
      };

      return {
        type:   sType as SupplyType,
        name:   currentSupply?.name ?? supplyLabel(sType as SupplyType),
        supply: currentSupply ?? fallbackSupply,
        points,
      };
    });

    return { printer, histories };
  }

  async function selectPrinter(idx: number) {
    selectedIdx = idx;
    const printer = $printers[idx];
    if (!printer) return;

    loading   = true;
    loadError = '';
    try {
      historyData[idx] = await loadHistory(printer);
      historyData = [...historyData];
    } catch {
      loadError = 'Не удалось загрузить историю';
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    if ($printers.length > 0) {
      historyData = $printers.map(() => null);
      await selectPrinter(0);
    }
  });

  $: selected = historyData[selectedIdx] ?? null;

  // ─── SVG helpers ──────────────────────────────────────────────────────────────

  function sparkPath(points: HistoryPoint[], w = 200, h = 40): string {
    if (points.length < 2) return '';
    const step = w / (points.length - 1);
    return points
      .map((p, i) => {
        const x = (i * step).toFixed(1);
        const y = (h - (p.pct / 100) * h).toFixed(1);
        return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
      })
      .join(' ');
  }

  function axisLabels(points: HistoryPoint[]): [string, string, string] {
    if (points.length === 0) return ['', '', 'Сейчас'];
    const fmt = (d: Date) =>
      d.toLocaleDateString('ru', { day: 'numeric', month: 'short' });
    return [
      fmt(points[0].date),
      fmt(points[Math.floor(points.length / 2)].date),
      'Сейчас',
    ];
  }
</script>

<Header />

<PageWrapper>
  <div class="history-layout">

    <!-- Список принтеров -->
    <div class="printer-list">
      {#each $printers as printer, i}
        <button
          class="printer-btn"
          class:printer-btn--active={selectedIdx === i}
          on:click={() => selectPrinter(i)}
        >
          <span class="printer-btn__name">{printer.name}</span>
          <span class="printer-btn__ip">{printer.ip}</span>
          {#if printer.supplies.some(s => s.isLow)}
            <span class="printer-btn__alert">!</span>
          {/if}
        </button>
      {:else}
        <p class="printer-list__empty">Принтеры не добавлены</p>
      {/each}
    </div>

    <!-- Графики -->
    <div class="charts">
      {#if loading}
        <div class="charts__state">
          <span class="charts__spinner">⟳</span>
          Загрузка истории...
        </div>

      {:else if loadError}
        <div class="charts__state charts__state--error">{loadError}</div>

      {:else if selected}
        <div class="charts__header">
          <h2 class="charts__title">{selected.printer.name}</h2>
          <span class="charts__sub">
            {selected.printer.model} · {selected.printer.ip}
          </span>
        </div>

        {#if selected.histories.length === 0}
          <div class="charts__state charts__state--empty">
            <span class="charts__state-icon">◷</span>
            <p>История пуста — данные появятся после первого опроса принтера</p>
          </div>
        {:else}
          <div class="chart-grid">
            {#each selected.histories as { supply, type, points } (type)}
              {@const color  = supplyTypeColor(type)}
              {@const path   = sparkPath(points)}
              {@const labels = axisLabels(points)}
              <Card padding="md">
                <div class="chart-card">

                  <div class="chart-card__header">
                    <div class="chart-card__dot" style="background:{color}"></div>
                    <span class="chart-card__name">{supplyLabel(type)}</span>
                    <span class="chart-card__pct" style="color:{tonerColor(supply.percent)}">
                      {supply.percent}%
                    </span>
                  </div>

                  <div class="chart-card__chart">
                    {#if path}
                      <svg viewBox="0 0 200 40" preserveAspectRatio="none" class="sparkline">
                        <defs>
                          <linearGradient id="grad-{type}" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%"   stop-color="{color}" stop-opacity="0.22" />
                            <stop offset="100%" stop-color="{color}" stop-opacity="0"    />
                          </linearGradient>
                        </defs>
                        <path
                          d="{path} L 200 40 L 0 40 Z"
                          fill="url(#grad-{type})"
                        />
                        <path
                          d="{path}"
                          fill="none"
                          stroke="{color}"
                          stroke-width="1.5"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        />
                      </svg>
                    {:else}
                      <span class="chart-card__nodata">недостаточно данных</span>
                    {/if}
                  </div>

                  <div class="chart-card__axis">
                    <span>{labels[0]}</span>
                    <span>{labels[1]}</span>
                    <span>{labels[2]}</span>
                  </div>

                  <div class="chart-card__stats">
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Мин.</span>
                      <span class="chart-card__stat-val">
                        {points.length ? Math.min(...points.map(p => p.pct)) : '—'}%
                      </span>
                    </div>
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Макс.</span>
                      <span class="chart-card__stat-val">
                        {points.length ? Math.max(...points.map(p => p.pct)) : '—'}%
                      </span>
                    </div>
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Сейчас</span>
                      <span
                        class="chart-card__stat-val"
                        style="color:{tonerColor(supply.percent)}"
                      >
                        {supply.percent}%
                      </span>
                    </div>
                  </div>

                </div>
              </Card>
            {/each}
          </div>
        {/if}

      {:else if $printers.length > 0}
        <!-- Принтеры загружаются, но история ещё не выбрана -->
        <div class="charts__state">Выберите принтер</div>
      {/if}
    </div>

  </div>
</PageWrapper>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .history-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    gap: v.$space-4;
    height: 100%;

    @include m.respond-below('md') {
      grid-template-columns: 1fr;
    }
  }

  // ── Printer list ──────────────────────────────────────────────────────────────

  .printer-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-content: start;

    &__empty {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      padding: v.$space-3;
      font-family: v.$font-mono;
    }
  }

  .printer-btn {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: v.$space-3;
    border-radius: v.$radius-md;
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    text-align: left;
    transition: all v.$transition-fast;
    @include m.focus-ring;

    &:hover { background: var(--nav-hover-bg); border-color: var(--border); }

    &--active { background: var(--nav-active-bg); border-color: var(--accent); }

    &__name {
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
      @include m.truncate;
      max-width: 170px;
    }

    &__ip {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__alert {
      position: absolute;
      top: v.$space-2;
      right: v.$space-2;
      width: 16px;
      height: 16px;
      border-radius: 50%;
      background: var(--status-warning);
      color: #000;
      font-size: 9px;
      font-weight: v.$font-weight-black;
      @include m.flex-center;
    }
  }

  // ── Charts area ───────────────────────────────────────────────────────────────

  .charts {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;
    min-width: 0;

    &__header {
      display: flex;
      align-items: baseline;
      gap: v.$space-3;
      flex-wrap: wrap;
    }

    &__title {
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }

    &__sub {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
    }

    &__state {
      @include m.flex-center;
      flex-direction: column;
      gap: v.$space-3;
      padding: v.$space-16;
      color: var(--text-tertiary);
      font-size: v.$font-size-sm;
      text-align: center;

      &--error {
        padding: v.$space-4;
        background: rgba(220, 53, 69, 0.08);
        border: 1px solid var(--status-error);
        border-radius: v.$radius-md;
        color: var(--status-error);
        flex-direction: row;
      }

      &--empty { flex-direction: column; }

      &-icon { font-size: 30px; opacity: 0.3; }
    }

    &__spinner {
      font-size: 26px;
      opacity: 0.4;
      animation: spin 1.2s linear infinite;
      display: block;
    }
  }

  // ── Chart grid ────────────────────────────────────────────────────────────────

  .chart-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: v.$space-4;
  }

  .chart-card {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;

    &__header {
      @include m.flex-start;
      gap: v.$space-2;
    }

    &__dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      flex-shrink: 0;
    }

    &__name {
      font-size: v.$font-size-xs;
      color: var(--text-secondary);
      font-weight: v.$font-weight-medium;
      flex: 1;
    }

    &__pct {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-bold;
    }

    &__chart {
      height: 52px;
      border-radius: v.$radius-md;
      overflow: hidden;
      background: var(--surface-2);
      @include m.flex-center;
    }

    &__nodata {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
      opacity: 0.5;
    }

    &__axis {
      @include m.flex-between;
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
    }

    &__stats {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: v.$space-2;
    }

    &__stat {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 1px;
    }

    &__stat-label {
      font-size: 9px;
      color: var(--text-tertiary);
      font-family: v.$font-mono;
    }

    &__stat-val {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }
  }

  .sparkline {
    width: 100%;
    height: 100%;
    display: block;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }
</style>
