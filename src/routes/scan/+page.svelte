<!-- src/routes/scan/+page.svelte -->

<script lang="ts">
  import { onDestroy } from 'svelte';
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { isScanning, scanResults, printers } from '$lib/stores/printers';
  import { notifications } from '$lib/stores/notifications';
  import { api } from '$lib/api/tauri';
  import type { ScanResult, PrinterBrand } from '$lib/types/printer';
  import type { UnlistenFn } from '$lib/api/tauri';

  let subnet = '192.168.1.0/24';
  let progress = 0;
  let currentIp = '';
  let scanLog: string[] = [];
  let unlistenProgress: UnlistenFn | null = null;

  async function startScan() {
    if ($isScanning) return;
    isScanning.set(true);
    scanResults.set([]);
    scanLog = [];
    progress = 0;
    currentIp = '';

    // Подписываемся на прогресс-события от бэкенда
    unlistenProgress = await api.onScanProgress(({ percent, current, found }) => {
      progress  = percent;
      currentIp = current;
      if (current) {
        scanLog = [
          `[${new Date().toLocaleTimeString()}] ${current} — проверяется...`,
          ...scanLog,
        ];
      }
    });

    try {
      const results: ScanResult[] = await api.scanNetwork(subnet);
      scanResults.set(results);

      // Дополняем лог только найденными принтерами
      const found = results.filter(r => r.isSNMPOpen);
      found.forEach(r => {
        scanLog = [
          `[${new Date().toLocaleTimeString()}] ${r.ip} — SNMP OK · ${r.sysDescr ?? r.model ?? 'Принтер'}`,
          ...scanLog,
        ];
      });

      notifications.success(
        'Сканирование завершено',
        `Найдено ${found.length} принтер${found.length === 1 ? '' : 'ов'}`
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Ошибка сканирования', msg);
    } finally {
      unlistenProgress?.();
      unlistenProgress = null;
      isScanning.set(false);
      progress = 0;
    }
  }

  function stopScan() {
    // Tauri команды не прерываются — просто сбрасываем UI
    // Реальная отмена потребует дополнительной команды cancel_scan в Rust
    unlistenProgress?.();
    unlistenProgress = null;
    isScanning.set(false);
    progress = 0;
    notifications.info('Сканирование прервано');
  }

  async function importPrinter(r: ScanResult) {
    try {
      const record = await api.addPrinter({
        ip:    r.ip,
        name:  `${r.model ?? 'Принтер'} (${r.ip})`,
        brand: r.brand ?? 'other',
        model: r.model ?? '',
      });
      // Добавляем в store с минимальными данными; poll придёт от планировщика
      printers.upsert({
        id:           record.id,
        ip:           record.ip,
        name:         record.name,
        brand:        record.brand as PrinterBrand,
        model:        record.model,
        location:     record.location,
        group:        record.group,
        status:       'unknown',
        supplies:     [],
        lastSeen:     new Date().toISOString(),
        addedManually: false,
      });
      notifications.success('Принтер добавлен', r.ip);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Не удалось добавить принтер', msg);
    }
  }

  onDestroy(() => {
    unlistenProgress?.();
  });

  $: knownIps = new Set($printers.map(p => p.ip));
</script>

<Header>
  <svelte:fragment slot="actions">
    {#if $isScanning}
      <Button variant="danger" size="sm" on:click={stopScan}>Остановить</Button>
    {:else}
      <Button variant="primary" size="sm" on:click={startScan}>▶ Запустить</Button>
    {/if}
  </svelte:fragment>
</Header>

<PageWrapper>
  <div class="scan-layout">
    <!-- Config panel -->
    <Card padding="md">
      <div class="config">
        <h3 class="config__title">Параметры сканирования</h3>

        <div class="config__form">
          <Input
            label="Подсеть (CIDR)"
            placeholder="192.168.1.0/24"
            bind:value={subnet}
            disabled={$isScanning}
          />
          <div class="config__hint">
            Будут опрошены все 254 хоста подсети по UDP/161 (SNMP).
            Community string задаётся в настройках.
          </div>
        </div>

        {#if $isScanning}
          <div class="config__progress-wrap">
            <div class="config__progress-header">
              <span class="config__progress-label">
                {currentIp ? currentIp : 'Сканирование...'}
              </span>
              <span class="config__progress-pct">{progress}%</span>
            </div>
            <div class="config__progress-track">
              <div class="config__progress-bar scan-indicator" style="width: {progress}%"></div>
            </div>
          </div>
        {/if}

        <!-- Stats -->
        <div class="config__stats">
          <div class="config__stat">
            <span class="config__stat-val">{$scanResults.length}</span>
            <span class="config__stat-lab">Найдено</span>
          </div>
          <div class="config__stat">
            <span class="config__stat-val">{$scanResults.filter(r => r.isSNMPOpen).length}</span>
            <span class="config__stat-lab">SNMP OK</span>
          </div>
          <div class="config__stat">
            <span class="config__stat-val">{$scanResults.filter(r => !knownIps.has(r.ip)).length}</span>
            <span class="config__stat-lab">Новых</span>
          </div>
        </div>
      </div>
    </Card>

    <!-- Results -->
    <div class="results">
      {#if $scanResults.length === 0 && !$isScanning}
        <div class="results__empty">
          <span class="results__empty-icon">◎</span>
          <p>Нажмите «Запустить» для поиска принтеров в сети</p>
        </div>
      {:else}
        <div class="result-list">
          {#each $scanResults as r (r.ip)}
            <div class="result-item animate-fade-in-up">
              <div class="result-item__left">
                <div class="result-item__indicator" class:result-item__indicator--ok={r.isSNMPOpen}></div>
                <div>
                  <div class="result-item__ip">{r.ip}</div>
                  {#if r.sysDescr}
                    <div class="result-item__desc">{r.sysDescr}</div>
                  {/if}
                </div>
              </div>
              <div class="result-item__right">
                {#if r.brand}
                  <Badge variant="default" size="sm">{r.brand}</Badge>
                {/if}
                {#if r.isSNMPOpen}
                  <Badge variant="success" size="sm" dot>SNMP</Badge>
                {:else}
                  <Badge variant="neutral" size="sm">Ping</Badge>
                {/if}
                {#if knownIps.has(r.ip)}
                  <Badge variant="info" size="sm">В списке</Badge>
                {:else if r.isSNMPOpen}
                  <Button variant="outline" size="sm" on:click={() => importPrinter(r)}>
                    + Добавить
                  </Button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Log -->
    {#if scanLog.length > 0}
      <Card padding="none">
        <div class="log">
          <div class="log__header">Лог сканирования</div>
          <div class="log__body">
            {#each scanLog as line}
              <div class="log__line">{line}</div>
            {/each}
          </div>
        </div>
      </Card>
    {/if}
  </div>
</PageWrapper>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .scan-layout {
    display: grid;
    grid-template-columns: 300px 1fr;
    grid-template-rows: auto 1fr auto;
    gap: v.$space-4;
    height: 100%;

    @include m.respond-below('lg') {
      grid-template-columns: 1fr;
    }

    & > :first-child { grid-row: 1 / 2; }
    & > :nth-child(2) { grid-column: 2; grid-row: 1 / 3; }
    & > :nth-child(3) { grid-column: 1 / -1; }

    @include m.respond-below('lg') {
      & > :nth-child(2) { grid-column: 1; grid-row: auto; }
    }
  }

  // Config
  .config {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;

    &__title {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-secondary);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-family: v.$font-mono;
    }

    &__form { display: flex; flex-direction: column; gap: v.$space-3; }

    &__hint {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      line-height: 1.5;
      padding: v.$space-2 v.$space-3;
      background: var(--surface-2);
      border-radius: v.$radius-md;
      border: 1px solid var(--border);
    }

    &__progress-wrap {
      display: flex;
      flex-direction: column;
      gap: v.$space-2;
    }
    &__progress-header { @include m.flex-between; }
    &__progress-label {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      font-family: v.$font-mono;
      @include m.truncate;
      max-width: 180px;
    }
    &__progress-pct {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      color: var(--accent);
      font-weight: v.$font-weight-bold;
    }
    &__progress-track {
      height: 3px;
      background: var(--gauge-track);
      border-radius: v.$radius-full;
      overflow: hidden;
    }
    &__progress-bar {
      height: 100%;
      background: var(--accent);
      border-radius: v.$radius-full;
      transition: width 0.3s ease;
      position: relative;
      overflow: hidden;
      &::after {
        content: '';
        position: absolute;
        inset: 0;
        background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
        animation: shimmer 1.2s ease-in-out infinite;
      }
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
      gap: 2px;
      padding: v.$space-2;
      background: var(--surface-2);
      border-radius: v.$radius-md;
      border: 1px solid var(--border);
    }
    &__stat-val {
      font-family: v.$font-display;
      font-size: v.$font-size-xl;
      font-weight: v.$font-weight-black;
      color: var(--accent);
      line-height: 1;
    }
    &__stat-lab {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
  }

  // Results
  .results {
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: v.$radius-lg;
    overflow: hidden;

    &__empty {
      @include m.flex-center;
      flex-direction: column;
      gap: v.$space-3;
      padding: v.$space-16;
      color: var(--text-tertiary);
      font-size: v.$font-size-sm;

      &-icon {
        font-size: 32px;
        opacity: 0.3;
      }
    }
  }

  .result-list { display: flex; flex-direction: column; }

  .result-item {
    @include m.flex-between;
    gap: v.$space-3;
    padding: v.$space-3 v.$space-4;
    border-bottom: 1px solid var(--border);
    transition: background v.$transition-fast;

    &:last-child { border-bottom: none; }
    &:hover { background: var(--nav-hover-bg); }

    &__left {
      @include m.flex-start;
      gap: v.$space-3;
      min-width: 0;
    }

    &__indicator {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--status-offline);
      flex-shrink: 0;
      &--ok { background: var(--status-online); }
    }

    &__ip {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
    }
    &__desc {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      margin-top: 1px;
    }

    &__right {
      @include m.flex-start;
      gap: v.$space-2;
      flex-shrink: 0;
    }
  }

  // Log
  .log {
    &__header {
      padding: v.$space-2 v.$space-4;
      border-bottom: 1px solid var(--border);
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    &__body {
      max-height: 140px;
      overflow-y: auto;
      padding: v.$space-2 v.$space-4;
      display: flex;
      flex-direction: column;
      gap: 2px;
      @include m.custom-scrollbar;
    }
    &__line {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
      line-height: 1.6;
    }
  }

  @keyframes shimmer {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX(300%); }
  }
</style>
