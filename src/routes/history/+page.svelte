<!-- src/routes/history/+page.svelte -->

<script lang="ts">
  import { onMount } from 'svelte';
  import Header        from '$lib/components/layout/Header.svelte';
  import PageWrapper   from '$lib/components/layout/PageWrapper.svelte';
  import Card          from '$lib/components/ui/Card.svelte';
  import Badge         from '$lib/components/ui/Badge.svelte';
  import Button        from '$lib/components/ui/Button.svelte';
  import SparklineChart from '$lib/components/charts/SparklineChart.svelte';
  import { printers }  from '$lib/stores/printers';
  import { supplyLabel, formatRelativeTime } from '$lib/utils/formatters';
  import { tonerColor, supplyTypeColor }      from '$lib/utils/colors';
  import { api, type SnapshotRecord }         from '$lib/api/tauri';
  import type { PrinterInfo, Supply, SupplyType } from '$lib/types/printer';

  // ─── Типы ────────────────────────────────────────────────────────────────────

  interface HistoryPoint { date: Date; pct: number; }

  interface SupplyHistory {
    type:   SupplyType;
    name:   string;
    supply: Supply;
    points: HistoryPoint[];
    // Статистика (вычисляется локально)
    min: number; max: number; avg: number;
    // Прогноз (вычисляется в SparklineChart, но нужен и здесь для Badge)
    forecastDays: number | null;
  }

  interface PrinterHistory {
    printer:   PrinterInfo;
    histories: SupplyHistory[];
    snapshots: SnapshotRecord[];
  }

  type PeriodKey = '7d' | '30d' | '90d' | 'all';

  const PERIODS: Array<{ key: PeriodKey; label: string; days: number }> = [
    { key: '7d',  label: '7 дней',  days: 7  },
    { key: '30d', label: '30 дней', days: 30 },
    { key: '90d', label: '90 дней', days: 90 },
    { key: 'all', label: 'Всё',     days: 0  },
  ];

  // ─── Состояние ───────────────────────────────────────────────────────────────

  let selectedIdx    = 0;
  let historyData: (PrinterHistory | null)[] = [];
  let loading        = false;
  let loadError      = '';
  let period: PeriodKey = '30d';
  let expandedTable: SupplyType | null = null; // раскрытая таблица
  let showAllRows    = false;

  const cache = new Map<string, SnapshotRecord[]>();

  // ─── Загрузка снапшотов ───────────────────────────────────────────────────────

  async function loadHistory(printer: PrinterInfo): Promise<PrinterHistory> {
    let snapshots: SnapshotRecord[] = [];

    // Загружаем максимально — фильтрация по периоду на фронте
    if (cache.has(printer.id)) {
      snapshots = cache.get(printer.id)!;
    } else {
      try {
        snapshots = await api.getSnapshots(printer.id, 365);
        cache.set(printer.id, snapshots);
      } catch (err) {
        console.error('[history] getSnapshots failed:', err);
      }
    }

    return buildHistory(printer, snapshots);
  }

  // ─── Построение SupplyHistory из снапшотов ────────────────────────────────────

  function buildHistory(printer: PrinterInfo, snapshots: SnapshotRecord[]): PrinterHistory {
    // Фильтруем по выбранному периоду
    const periodDays = PERIODS.find(p => p.key === period)?.days ?? 0;
    const cutoff     = periodDays > 0
      ? new Date(Date.now() - periodDays * 86400000)
      : new Date(0);

    const filtered = snapshots.filter(s => new Date(s.timestamp) >= cutoff);

    // Собираем все типы расходников
    const supplyTypes = new Set<string>();
    printer.supplies.forEach(s => supplyTypes.add(s.type));
    filtered.forEach(snap => {
      try {
        const arr: Array<{ type: string }> = JSON.parse(snap.suppliesJson);
        arr.forEach(s => supplyTypes.add(s.type));
      } catch { /* битая запись */ }
    });

    const histories: SupplyHistory[] = Array.from(supplyTypes).map(sType => {
      const currentSupply = printer.supplies.find(s => s.type === sType);

      const points: HistoryPoint[] = filtered
        .map(snap => {
          try {
            const arr: Array<{ type: string; percent: number }> =
              JSON.parse(snap.suppliesJson);
            const found = arr.find(s => s.type === sType);
            if (!found) return null;
            return { date: new Date(snap.timestamp), pct: found.percent };
          } catch { return null; }
        })
        .filter((p): p is HistoryPoint => p !== null)
        .sort((a, b) => a.date.getTime() - b.date.getTime());

      // Добавляем текущую точку если есть расходник
      if (currentSupply) {
        const last = points.at(-1);
        // Добавляем только если последняя точка старше 5 минут
        if (!last || Date.now() - last.date.getTime() > 5 * 60 * 1000) {
          points.push({ date: new Date(), pct: currentSupply.percent });
        }
      }

      // Статистика
      const pcts = points.map(p => p.pct);
      const min  = pcts.length ? Math.min(...pcts) : 0;
      const max  = pcts.length ? Math.max(...pcts) : 0;
      const avg  = pcts.length ? Math.round(pcts.reduce((a, b) => a + b, 0) / pcts.length) : 0;

      // Прогноз (тот же алгоритм что в SparklineChart, для Badge)
      const forecastDays = computeForecastDays(points);

      const fallbackSupply: Supply = {
        type:       sType as SupplyType,
        name:       supplyLabel(sType as SupplyType),
        level:      0, maxLevel: 100,
        percent:    points.at(-1)?.pct ?? 0,
        isLow:      false, isCritical: false,
      };

      return {
        type:    sType as SupplyType,
        name:    currentSupply?.name ?? supplyLabel(sType as SupplyType),
        supply:  currentSupply ?? fallbackSupply,
        points, min, max, avg, forecastDays,
      };
    });

    // Сортируем: тонеры первыми, потом барабан, прочее
    const order: Record<string, number> = {
      toner_black: 0, toner_cyan: 1, toner_magenta: 2, toner_yellow: 3,
      drum: 4, fuser: 5, waste: 6, other: 7,
    };
    histories.sort((a, b) => (order[a.type] ?? 9) - (order[b.type] ?? 9));

    return { printer, histories, snapshots: filtered };
  }

  // ─── Линейный прогноз (упрощённый, для Badge) ────────────────────────────────

  function computeForecastDays(points: HistoryPoint[]): number | null {
    if (points.length < 3) return null;
    const slice = points.slice(-Math.min(30, points.length));
    const n = slice.length;
    const t0 = slice[0].date.getTime();
    const xs = slice.map(p => (p.date.getTime() - t0) / 86400000);
    const ys = slice.map(p => p.pct);
    const sumX  = xs.reduce((a, b) => a + b, 0);
    const sumY  = ys.reduce((a, b) => a + b, 0);
    const sumXY = xs.reduce((a, x, i) => a + x * ys[i], 0);
    const sumX2 = xs.reduce((a, x) => a + x * x, 0);
    const d = n * sumX2 - sumX * sumX;
    if (Math.abs(d) < 1e-9) return null;
    const slope = (n * sumXY - sumX * sumY) / d;
    const intercept = (sumY - slope * sumX) / n;
    if (slope >= -0.01) return null;
    const lastX = xs[xs.length - 1];
    const days  = Math.round((0 - intercept) / slope - lastX);
    return days < 0 ? 0 : days;
  }

  // ─── Выбор принтера ───────────────────────────────────────────────────────────

  async function selectPrinter(idx: number) {
    selectedIdx = idx;
    const printer = $printers[idx];
    if (!printer) return;
    loading   = true;
    loadError = '';
    expandedTable = null;
    showAllRows   = false;
    try {
      historyData[idx] = await loadHistory(printer);
      historyData      = [...historyData];
    } catch {
      loadError = 'Не удалось загрузить историю';
    } finally {
      loading = false;
    }
  }

  // ─── При смене периода — перестроить без перезапроса БД ───────────────────────

  function onPeriodChange(key: PeriodKey) {
    period = key;
    const idx = selectedIdx;
    const printer = $printers[idx];
    if (!printer) return;
    // Берём из кеша, просто пересчитываем фильтр
    const cached = cache.get(printer.id);
    if (cached) {
      historyData[idx] = buildHistory(printer, cached);
      historyData = [...historyData];
    }
  }

  onMount(() => {
    if ($printers.length > 0) {
      historyData = $printers.map(() => null);
      selectPrinter(0);
    }
  });

  $: selected = historyData[selectedIdx] ?? null;

  // ─── Экспорт CSV ──────────────────────────────────────────────────────────────

  function exportCsv() {
    if (!selected) return;
    const rows: string[][] = [
      ['Принтер', 'Дата', 'Расходник', 'Тип', 'Уровень (%)'],
    ];
    for (const h of selected.histories) {
      for (const pt of h.points) {
        rows.push([
          selected.printer.name,
          pt.date.toISOString(),
          h.name,
          h.type,
          pt.pct.toString(),
        ]);
      }
    }
    const csv   = rows.map(r => r.map(c => `"${c.replace(/"/g, '""')}"`).join(',')).join('\n');
    const blob  = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8;' });
    const url   = URL.createObjectURL(blob);
    const a     = document.createElement('a');
    a.href      = url;
    a.download  = `tonerscope-${selected.printer.name.replace(/\s+/g, '_')}-${period}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<Header />

<PageWrapper>
  <div class="history-layout">

    <!-- ── Список принтеров ────────────────────────────────────────────────── -->
    <div class="printer-list">
      {#each $printers as printer, i}
        {@const lowCount = printer.supplies.filter(s => s.isLow).length}
        <button
          class="printer-btn"
          class:printer-btn--active={selectedIdx === i}
          on:click={() => selectPrinter(i)}
        >
          <span class="printer-btn__name">{printer.name}</span>
          <span class="printer-btn__ip">{printer.ip}</span>
          {#if lowCount > 0}
            <span class="printer-btn__alert">{lowCount}</span>
          {/if}
        </button>
      {:else}
        <p class="printer-list__empty">Принтеры не добавлены</p>
      {/each}
    </div>

    <!-- ── Область графиков ───────────────────────────────────────────────── -->
    <div class="charts">

      {#if loading}
        <div class="charts__state">
          <span class="charts__spinner">⟳</span>
          Загрузка истории...
        </div>

      {:else if loadError}
        <div class="charts__state charts__state--error">{loadError}</div>

      {:else if selected}

        <!-- Шапка с заголовком, фильтрами и кнопкой экспорта -->
        <div class="charts__header">
          <div class="charts__title-row">
            <h2 class="charts__title">{selected.printer.name}</h2>
            <span class="charts__sub">
              {selected.printer.model} · {selected.printer.ip}
            </span>
          </div>

          <div class="charts__controls">
            <!-- Фильтр периода -->
            <div class="period-filter" role="group" aria-label="Фильтр периода">
              {#each PERIODS as p}
                <button
                  class="period-btn"
                  class:period-btn--active={period === p.key}
                  on:click={() => onPeriodChange(p.key)}
                >
                  {p.label}
                </button>
              {/each}
            </div>

            <!-- Экспорт CSV -->
            <Button variant="ghost" on:click={exportCsv} title="Экспорт в CSV">
              ↓ CSV
            </Button>
          </div>
        </div>

        {#if selected.histories.length === 0}
          <div class="charts__state charts__state--empty">
            <span class="charts__state-icon">◷</span>
            <p>История пуста — данные появятся после первого опроса принтера</p>
          </div>

        {:else}

          <!-- Сводная строка по принтеру -->
          <div class="summary-bar">
            <div class="summary-bar__stat">
              <span class="summary-bar__label">Снапшотов</span>
              <span class="summary-bar__val">{selected.snapshots.length}</span>
            </div>
            <div class="summary-bar__stat">
              <span class="summary-bar__label">Расходников</span>
              <span class="summary-bar__val">{selected.histories.length}</span>
            </div>
            {#if selected.printer.pageCount}
              <div class="summary-bar__stat">
                <span class="summary-bar__label">Страниц</span>
                <span class="summary-bar__val">{selected.printer.pageCount.toLocaleString('ru')}</span>
              </div>
            {/if}
            <div class="summary-bar__stat">
              <span class="summary-bar__label">Обновлено</span>
              <span class="summary-bar__val">{formatRelativeTime(selected.printer.lastSeen)}</span>
            </div>
          </div>

          <!-- Сетка карточек графиков -->
          <div class="chart-grid">
            {#each selected.histories as h (h.type)}
              {@const color = supplyTypeColor(h.type)}
              <Card padding="md">
                <div class="chart-card">

                  <!-- Заголовок карточки -->
                  <div class="chart-card__header">
                    <div class="chart-card__dot" style="background:{color}"></div>
                    <span class="chart-card__name">{supplyLabel(h.type)}</span>
                    <span class="chart-card__pct" style="color:{tonerColor(h.supply.percent)}">
                      {h.supply.percent}%
                    </span>
                    {#if h.supply.isCritical}
                      <Badge variant="error" dot>Крит.</Badge>
                    {:else if h.supply.isLow}
                      <Badge variant="warning" dot>Низко</Badge>
                    {/if}
                  </div>

                  <!-- Интерактивный график -->
                  <div class="chart-card__chart">
                    <SparklineChart
                      points={h.points}
                      {color}
                      supplyType={h.type}
                    />
                  </div>

                  <!-- Статистика под графиком -->
                  <div class="chart-card__stats">
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Мин.</span>
                      <span class="chart-card__stat-val">{h.min}%</span>
                    </div>
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Сред.</span>
                      <span class="chart-card__stat-val">{h.avg}%</span>
                    </div>
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Макс.</span>
                      <span class="chart-card__stat-val">{h.max}%</span>
                    </div>
                    <div class="chart-card__stat">
                      <span class="chart-card__stat-label">Сейчас</span>
                      <span
                        class="chart-card__stat-val"
                        style="color:{tonerColor(h.supply.percent)}"
                      >{h.supply.percent}%</span>
                    </div>
                  </div>

                  <!-- Кнопка раскрытия таблицы -->
                  {#if h.points.length > 0}
                    <button
                      class="chart-card__table-toggle"
                      on:click={() => {
                        if (expandedTable === h.type) {
                          expandedTable = null;
                        } else {
                          expandedTable = h.type;
                          showAllRows   = false;
                        }
                      }}
                    >
                      {expandedTable === h.type ? '▲ Скрыть' : '▾ Показать данные'}
                    </button>
                  {/if}

                  <!-- Раскрытая таблица -->
                  {#if expandedTable === h.type}
                    <div class="history-table-wrap">
                      <table class="history-table">
                        <thead>
                          <tr>
                            <th>Дата</th>
                            <th>Уровень</th>
                            <th>Δ</th>
                          </tr>
                        </thead>
                        <tbody>
                          {#each (showAllRows ? [...h.points].reverse() : [...h.points].reverse().slice(0, 10)) as pt, i}
                            {@const prev = h.points[h.points.length - 1 - i - 1]}
                            {@const delta = prev ? pt.pct - prev.pct : null}
                            <tr>
                              <td class="history-table__date">
                                {pt.date.toLocaleString('ru', {
                                  day:'2-digit', month:'2-digit',
                                  hour:'2-digit', minute:'2-digit'
                                })}
                              </td>
                              <td class="history-table__pct" style="color:{tonerColor(pt.pct)}">
                                {pt.pct}%
                              </td>
                              <td class="history-table__delta">
                                {#if delta !== null}
                                  <span class:delta--up={delta > 0} class:delta--down={delta < 0}>
                                    {delta > 0 ? '+' : ''}{delta}%
                                  </span>
                                {:else}
                                  <span class="history-table__na">—</span>
                                {/if}
                              </td>
                            </tr>
                          {/each}
                        </tbody>
                      </table>
                      {#if !showAllRows && h.points.length > 10}
                        <button
                          class="chart-card__table-toggle"
                          on:click={() => showAllRows = true}
                        >
                          Показать все {h.points.length} записей
                        </button>
                      {/if}
                    </div>
                  {/if}

                </div>
              </Card>
            {/each}
          </div>
        {/if}

      {:else if $printers.length > 0}
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
      align-items: flex-start;
      justify-content: space-between;
      gap: v.$space-3;
      flex-wrap: wrap;
    }

    &__title-row {
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

    &__controls {
      @include m.flex-start;
      gap: v.$space-2;
      flex-wrap: wrap;
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
        background: rgba(220,53,69,.08);
        border: 1px solid var(--status-error);
        border-radius: v.$radius-md;
        color: var(--status-error);
        flex-direction: row;
      }

      &--empty { flex-direction: column; }
      &-icon   { font-size: 30px; opacity: 0.3; }
    }

    &__spinner {
      font-size: 26px;
      opacity: 0.4;
      animation: spin 1.2s linear infinite;
      display: block;
    }
  }

  // ── Period filter ─────────────────────────────────────────────────────────────

  .period-filter {
    @include m.flex-start;
    gap: 2px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: v.$radius-md;
    padding: 2px;
  }

  .period-btn {
    padding: 4px v.$space-3;
    border-radius: calc(v.$radius-md - 2px);
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: v.$font-mono;
    font-size: v.$font-size-xs;
    color: var(--text-secondary);
    transition: all v.$transition-fast;
    @include m.focus-ring;

    &:hover { color: var(--text-primary); }

    &--active {
      background: var(--accent);
      color: var(--bg);
      font-weight: v.$font-weight-semibold;
    }
  }

  // ── Summary bar ───────────────────────────────────────────────────────────────

  .summary-bar {
    @include m.flex-start;
    gap: v.$space-6;
    padding: v.$space-3 v.$space-4;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: v.$radius-md;
    flex-wrap: wrap;

    &__stat {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }

    &__label {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }

    &__val {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }
  }

  // ── Chart grid ────────────────────────────────────────────────────────────────

  .chart-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: v.$space-4;
  }

  .chart-card {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;

    &__header {
      @include m.flex-start;
      gap: v.$space-2;
      flex-wrap: wrap;
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

    &__stats {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: v.$space-1;
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

    &__table-toggle {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
      background: none;
      border: none;
      cursor: pointer;
      padding: v.$space-1 0;
      @include m.focus-ring;

      &:hover { color: var(--accent); }
    }
  }

  // ── History table ─────────────────────────────────────────────────────────────

  .history-table-wrap {
    border-top: 1px solid var(--border);
    padding-top: v.$space-2;
    max-height: 240px;
    overflow-y: auto;
    @include m.custom-scrollbar(4px);
  }

  .history-table {
    width: 100%;
    border-collapse: collapse;
    font-family: v.$font-mono;
    font-size: 10px;

    th {
      color: var(--text-tertiary);
      font-weight: v.$font-weight-regular;
      text-align: left;
      padding: 3px v.$space-2;
      border-bottom: 1px solid var(--border);
      position: sticky;
      top: 0;
      background: var(--surface-1);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      font-size: 8px;
    }

    td {
      padding: 4px v.$space-2;
      border-bottom: 1px solid rgba(255,255,255,0.03);
      vertical-align: middle;
    }

    tr:last-child td { border-bottom: none; }

    tr:hover td { background: var(--surface-2); }

    &__date { color: var(--text-secondary); }

    &__pct { font-weight: v.$font-weight-bold; }

    &__delta { text-align: right; }

    &__na { color: var(--text-tertiary); opacity: 0.5; }
  }

  .delta {
    &--up   { color: var(--status-online); }
    &--down { color: var(--status-warning); }
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }
</style>
