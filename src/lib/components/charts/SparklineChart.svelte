<!-- src/lib/components/charts/SparklineChart.svelte -->
<!--
  Интерактивный SVG-график уровня расходника.
  - Hover: вертикальный курсор + tooltip с датой и значением
  - Прогноз: линейная экстраполяция пунктирной линией до 0%
  - Зоны: фоновая подсветка критической и низкой зон (<10%, <20%)
  - Анимация: плавное появление линии через stroke-dashoffset
-->
<script lang="ts">
  import type { SupplyType } from '$lib/types/printer';

  // ─── Props ────────────────────────────────────────────────────────────────────

  export let points:    Array<{ date: Date; pct: number }> = [];
  export let color:     string  = 'var(--accent)';
  export let supplyType: SupplyType | string = 'other';

  // Прогноз: сколько дней вперёд экстраполировать (0 = авто до 0%)
  export let forecastDays: number = 0;

  // ─── Размеры SVG (viewBox) ────────────────────────────────────────────────────

  const W = 300;
  const H = 80;
  const PAD_L = 0;
  const PAD_R = forecastDays > 0 ? 40 : 0; // место для прогноза

  // ─── Состояние hover ──────────────────────────────────────────────────────────

  let hoverIdx:  number | null = null;
  let tooltipX = 0;
  let tooltipY = 0;

  // ─── Вычисления ───────────────────────────────────────────────────────────────

  $: sortedPoints = [...points].sort((a, b) => a.date.getTime() - b.date.getTime());

  $: effectiveW = W - PAD_L - PAD_R;

  // Координата X по индексу точки
  function xOf(i: number, total: number): number {
    if (total < 2) return PAD_L + effectiveW / 2;
    return PAD_L + (i / (total - 1)) * effectiveW;
  }

  // Координата Y по проценту (0% = низ, 100% = верх)
  function yOf(pct: number): number {
    return H - (Math.max(0, Math.min(100, pct)) / 100) * H;
  }

  // Построить path из точек
  $: linePath = (() => {
    if (sortedPoints.length < 2) return '';
    return sortedPoints
      .map((p, i) => {
        const x = xOf(i, sortedPoints.length).toFixed(2);
        const y = yOf(p.pct).toFixed(2);
        return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
      })
      .join(' ');
  })();

  // Путь заливки под линией
  $: fillPath = (() => {
    if (sortedPoints.length < 2) return '';
    const n = sortedPoints.length;
    const lastX = xOf(n - 1, n).toFixed(2);
    return `${linePath} L ${lastX} ${H} L ${PAD_L} ${H} Z`;
  })();

  // ─── Линейный прогноз ─────────────────────────────────────────────────────────

  interface Forecast {
    path:         string;
    fillPath:     string;
    daysLeft:     number | null;
    endX:         number;
    endY:         number;
  }

  $: forecast = (() : Forecast | null => {
    if (sortedPoints.length < 3) return null;

    // Метод наименьших квадратов по последним min(30, N) точкам
    const slice = sortedPoints.slice(-Math.min(30, sortedPoints.length));
    const n = slice.length;
    const t0 = slice[0].date.getTime();
    const xs = slice.map(p => (p.date.getTime() - t0) / 86400000); // в днях
    const ys = slice.map(p => p.pct);

    const sumX  = xs.reduce((a, b) => a + b, 0);
    const sumY  = ys.reduce((a, b) => a + b, 0);
    const sumXY = xs.reduce((a, xi, i) => a + xi * ys[i], 0);
    const sumX2 = xs.reduce((a, xi) => a + xi * xi, 0);

    const denom = n * sumX2 - sumX * sumX;
    if (Math.abs(denom) < 1e-9) return null;

    const slope     = (n * sumXY - sumX * sumY) / denom;
    const intercept = (sumY - slope * sumX) / n;

    // Сколько дней до 0% (slope < 0)
    let daysLeft: number | null = null;
    if (slope < -0.01) {
      const lastXDays = xs[xs.length - 1];
      daysLeft = Math.round((0 - intercept) / slope - lastXDays);
      if (daysLeft < 0) daysLeft = 0;
    }

    // Строим прогноз вперёд
    const lastPoint  = sortedPoints[sortedPoints.length - 1];
    const lastXDays  = xs[xs.length - 1];
    const extendDays = forecastDays > 0
      ? forecastDays
      : daysLeft != null
        ? Math.min(daysLeft + 5, 60)
        : 30;

    if (extendDays <= 0) return null;

    // Начало прогноза — последняя реальная точка
    const startX = xOf(sortedPoints.length - 1, sortedPoints.length);
    const startY = yOf(lastPoint.pct);

    // Конец прогноза на экране
    // Растягиваем прогноз за правый край (в PAD_R зону)
    const totalDaysOnScreen = extendDays;
    const extendPixels = PAD_R > 0
      ? PAD_R
      : (effectiveW / Math.max(1, sortedPoints.length - 1)) * Math.min(extendDays, 10);

    const endXFull = startX + extendPixels;
    const endPct   = Math.max(0, intercept + slope * (lastXDays + extendDays));
    const endY     = yOf(endPct);

    const fPath      = `M ${startX.toFixed(2)} ${startY.toFixed(2)} L ${endXFull.toFixed(2)} ${endY.toFixed(2)}`;
    const fFillPath  = `M ${startX.toFixed(2)} ${H} L ${startX.toFixed(2)} ${startY.toFixed(2)} L ${endXFull.toFixed(2)} ${endY.toFixed(2)} L ${endXFull.toFixed(2)} ${H} Z`;

    return { path: fPath, fillPath: fFillPath, daysLeft, endX: endXFull, endY };
  })();

  // ─── Длина линии для анимации ─────────────────────────────────────────────────

  let lineLength = 1000; // резервный; пересчитывается через bind:this

  // ─── Hover-логика ─────────────────────────────────────────────────────────────

  function onMouseMove(e: MouseEvent) {
    const svg  = (e.currentTarget as SVGElement);
    const rect = svg.getBoundingClientRect();
    const rawX = ((e.clientX - rect.left) / rect.width) * W;

    if (sortedPoints.length === 0) { hoverIdx = null; return; }
    if (sortedPoints.length === 1) { hoverIdx = 0; return; }

    // Найти ближайшую точку по X
    let closest = 0;
    let minDist  = Infinity;
    sortedPoints.forEach((_, i) => {
      const d = Math.abs(xOf(i, sortedPoints.length) - rawX);
      if (d < minDist) { minDist = d; closest = i; }
    });

    hoverIdx = closest;
    tooltipX = xOf(closest, sortedPoints.length);
    tooltipY = yOf(sortedPoints[closest].pct);
  }

  function onMouseLeave() { hoverIdx = null; }

  // ─── Форматирование ───────────────────────────────────────────────────────────

  function fmtDate(d: Date): string {
    return d.toLocaleDateString('ru', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
  }

  function fmtDateShort(d: Date): string {
    return d.toLocaleDateString('ru', { day: 'numeric', month: 'short' });
  }

  // Tooltip по X: слева или справа
  $: tooltipOnRight = hoverIdx !== null && tooltipX < W * 0.6;

  // ID для уникальных defs (несколько графиков на странице)
  const uid = supplyType + '-' + Math.random().toString(36).slice(2, 7);
</script>

<div class="sparkline-wrap">
  {#if sortedPoints.length < 2}
    <div class="sparkline-empty">
      <span class="sparkline-empty__icon">◷</span>
      <span>недостаточно данных</span>
    </div>
  {:else}
    <svg
      class="sparkline-svg"
      viewBox="0 0 {W} {H}"
      preserveAspectRatio="none"
      role="img"
      aria-label="График уровня расходника"
      on:mousemove={onMouseMove}
      on:mouseleave={onMouseLeave}
    >
      <defs>
        <!-- Градиент заливки основной линии -->
        <linearGradient id="fill-{uid}" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stop-color={color} stop-opacity="0.18" />
          <stop offset="100%" stop-color={color} stop-opacity="0"    />
        </linearGradient>

        <!-- Градиент заливки прогноза -->
        <linearGradient id="fill-fc-{uid}" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stop-color={color} stop-opacity="0.08" />
          <stop offset="100%" stop-color={color} stop-opacity="0"    />
        </linearGradient>

        <!-- Clip для основного контента (без выхода за границы) -->
        <clipPath id="clip-{uid}">
          <rect x="0" y="0" width={W} height={H} />
        </clipPath>
      </defs>

      <!-- Фоновые зоны (критическая < 10%, низкая 10–20%) -->
      <rect
        x="0" y={yOf(10)} width={W} height={H - yOf(10)}
        fill="var(--status-error)" opacity="0.05"
      />
      <rect
        x="0" y={yOf(20)} width={W} height={yOf(10) - yOf(20)}
        fill="var(--status-warning)" opacity="0.04"
      />

      <!-- Горизонтальные линии-ориентиры -->
      {#each [20, 50, 80] as pctLine}
        <line
          x1={PAD_L} y1={yOf(pctLine)}
          x2={W}     y2={yOf(pctLine)}
          stroke="var(--border)"
          stroke-width="0.5"
          stroke-dasharray="2,3"
          opacity="0.5"
        />
      {/each}

      <g clip-path="url(#clip-{uid})">
        <!-- Заливка под линией прогноза -->
        {#if forecast}
          <path
            d={forecast.fillPath}
            fill="url(#fill-fc-{uid})"
          />
        {/if}

        <!-- Заливка под основной линией -->
        <path d={fillPath} fill="url(#fill-{uid})" />

        <!-- Линия прогноза (пунктир) -->
        {#if forecast}
          <path
            d={forecast.path}
            fill="none"
            stroke={color}
            stroke-width="1.2"
            stroke-dasharray="3,3"
            stroke-linecap="round"
            opacity="0.55"
          />
          <!-- Точка конца прогноза -->
          <circle
            cx={forecast.endX}
            cy={forecast.endY}
            r="2.5"
            fill={color}
            opacity="0.4"
          />
        {/if}

        <!-- Основная линия -->
        <path
          class="sparkline-line"
          d={linePath}
          fill="none"
          stroke={color}
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        />

        <!-- Точки на линии (только при hover) -->
        {#each sortedPoints as p, i}
          <circle
            class="sparkline-dot"
            class:sparkline-dot--hover={hoverIdx === i}
            cx={xOf(i, sortedPoints.length)}
            cy={yOf(p.pct)}
            r={hoverIdx === i ? 3.5 : 2}
            fill={color}
            stroke="var(--surface-1)"
            stroke-width="1.5"
          />
        {/each}

        <!-- Курсор-линия при hover -->
        {#if hoverIdx !== null}
          <line
            x1={tooltipX} y1="0"
            x2={tooltipX} y2={H}
            stroke={color}
            stroke-width="1"
            stroke-dasharray="2,2"
            opacity="0.5"
          />
        {/if}
      </g>

      <!-- Метки оси X (крайние + середина) -->
      {#if sortedPoints.length >= 2}
        {@const first = sortedPoints[0]}
        {@const last  = sortedPoints[sortedPoints.length - 1]}
        <text
          x={PAD_L + 2}
          y={H - 2}
          class="sparkline-axis-label"
          text-anchor="start"
        >{fmtDateShort(first.date)}</text>
        <text
          x={PAD_L + effectiveW - 2}
          y={H - 2}
          class="sparkline-axis-label"
          text-anchor="end"
        >сейчас</text>
      {/if}
    </svg>

    <!-- Tooltip (вне SVG для правильного overflow) -->
    {#if hoverIdx !== null}
      {@const pt = sortedPoints[hoverIdx]}
      {@const txPct = (tooltipX / W) * 100}
      <div
        class="sparkline-tooltip"
        class:sparkline-tooltip--right={tooltipOnRight}
        style="left:{txPct}%"
        role="tooltip"
      >
        <span class="sparkline-tooltip__date">{fmtDate(pt.date)}</span>
        <span class="sparkline-tooltip__val" style="color:{color}">{pt.pct}%</span>
      </div>
    {/if}

    <!-- Прогноз-бейдж -->
    {#if forecast?.daysLeft !== null && forecast?.daysLeft !== undefined}
      <div
        class="sparkline-forecast"
        class:sparkline-forecast--critical={forecast.daysLeft <= 7}
        class:sparkline-forecast--warn={forecast.daysLeft > 7 && forecast.daysLeft <= 20}
      >
        {#if forecast.daysLeft === 0}
          <span class="sparkline-forecast__icon">⚠</span> Тонер заканчивается
        {:else}
          <span class="sparkline-forecast__icon">◈</span>
          Прогноз: ~{forecast.daysLeft} дн.
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .sparkline-wrap {
    position: relative;
    width: 100%;
  }

  // ── SVG ───────────────────────────────────────────────────────────────────────

  .sparkline-svg {
    display: block;
    width: 100%;
    height: 80px;
    cursor: crosshair;
    overflow: visible;
  }

  :global(.sparkline-axis-label) {
    font-family: v.$font-mono;
    font-size: 7px;
    fill: var(--text-tertiary);
  }

  .sparkline-dot {
    transition: r v.$transition-fast;
    pointer-events: none;
    opacity: 0;

    &--hover { opacity: 1; }
  }

  // Анимация появления линии
  .sparkline-line {
    stroke-dasharray: 2000;
    stroke-dashoffset: 2000;
    animation: draw-line 0.7s cubic-bezier(0.4, 0, 0.2, 1) forwards;
  }

  @keyframes draw-line {
    to { stroke-dashoffset: 0; }
  }

  // ── Tooltip ───────────────────────────────────────────────────────────────────

  .sparkline-tooltip {
    position: absolute;
    top: -6px;
    transform: translateX(-50%) translateY(-100%);
    background: var(--surface-3, var(--surface-2));
    border: 1px solid var(--border);
    border-radius: v.$radius-md;
    padding: 4px v.$space-2;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    pointer-events: none;
    white-space: nowrap;
    z-index: v.$z-dropdown;
    box-shadow: 0 4px 16px rgba(0,0,0,0.25);
    animation: tooltip-in 80ms ease both;

    &--right {
      transform: translateX(-8px) translateY(-100%);
      left: auto !important;
    }

    &__date {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
    }

    &__val {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-bold;
    }
  }

  @keyframes tooltip-in {
    from { opacity: 0; transform: translateX(-50%) translateY(calc(-100% + 4px)); }
    to   { opacity: 1; transform: translateX(-50%) translateY(-100%); }
  }

  // ── Прогноз-бейдж ─────────────────────────────────────────────────────────────

  .sparkline-forecast {
    @include m.flex-start;
    gap: 4px;
    margin-top: v.$space-1;
    font-family: v.$font-mono;
    font-size: 9px;
    color: var(--text-tertiary);

    &__icon { font-size: 9px; }

    &--warn {
      color: var(--status-warning);
    }

    &--critical {
      color: var(--status-error);
      font-weight: v.$font-weight-bold;
      animation: pulse-text 1.4s ease-in-out infinite;
    }
  }

  @keyframes pulse-text {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.55; }
  }

  // ── Empty state ───────────────────────────────────────────────────────────────

  .sparkline-empty {
    @include m.flex-center;
    gap: v.$space-2;
    height: 80px;
    font-family: v.$font-mono;
    font-size: 9px;
    color: var(--text-tertiary);
    opacity: 0.45;

    &__icon { font-size: 18px; opacity: 0.5; }
  }
</style>
