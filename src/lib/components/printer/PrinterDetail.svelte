<!-- src/lib/components/printer/PrinterDetail.svelte -->

<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import type { PrinterInfo } from '$lib/types/printer';
  import StatusBadge from './StatusBadge.svelte';
  import TonerGauge from './TonerGauge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { formatPageCount, formatRelativeTime, brandLabel } from '$lib/utils/formatters';
  import { notifications } from '$lib/stores/notifications';
  import { printers, mergeSnapshot } from '$lib/stores/printers';
  import { api } from '$lib/api/tauri';

  export let printer: PrinterInfo;

  const dispatch = createEventDispatcher<{ close: void }>();

  let polling        = false;
  let removing       = false;
  let restartingSpooler = false;

  // true — Windows, false — другие ОС или не определено
  let isWindows = false;
  // Текущий статус службы Spooler
  let spoolerStatus = '';

  onMount(() => {
    // Определяем платформу и запрашиваем статус спулера (не async onMount — паттерн проекта)
    api.getSpoolerStatus()
      .then((status) => {
        isWindows     = status !== 'unavailable';
        spoolerStatus = status;
      })
      .catch(() => {
        isWindows     = false;
        spoolerStatus = 'unavailable';
      });
  });

  async function pollNow() {
    polling = true;
    try {
      const snap = await api.pollPrinter(printer.ip);
      printers.upsert(mergeSnapshot(printer, snap));
      notifications.success('Опрос выполнен', `${printer.name} обновлён`);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Ошибка опроса', msg);
    } finally {
      polling = false;
    }
  }

  async function removePrinter() {
    removing = true;
    try {
      await api.removePrinter(printer.id);
      printers.remove(printer.id);
      dispatch('close');
      notifications.info('Принтер удалён', printer.name);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Не удалось удалить', msg);
      removing = false;
    }
  }

  async function restartSpooler() {
    restartingSpooler = true;
    spoolerStatus     = 'stop_pending';
    try {
      const result = await api.restartSpooler();
      spoolerStatus = result.status;
      if (result.success) {
        notifications.success('Print Spooler перезапущен', result.message);
      } else {
        notifications.warning('Спулер перезапускается', result.message);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Ошибка перезапуска спулера', msg);
      spoolerStatus = 'unknown';
    } finally {
      restartingSpooler = false;
    }
  }

  const brandIconMap: Record<string, string> = {
    pantum: 'P', kyocera: 'K', hp: 'H', canon: 'C', other: '?',
  };

  // Человекочитаемый статус службы
  const spoolerStatusLabels: Record<string, string> = {
    running:       'Запущен',
    stopped:       'Остановлен',
    start_pending: 'Запускается…',
    stop_pending:  'Останавливается…',
    unknown:       'Неизвестно',
    unavailable:   '',
  };
</script>

<aside
  class="detail"
  transition:fly={{ x: 24, duration: 260 }}
>
  <!-- Header -->
  <div class="detail__header">
    <div class="detail__brand-badge" data-brand={printer.brand}>
      {brandIconMap[printer.brand] ?? '?'}
    </div>
    <div class="detail__title-block">
      <h2 class="detail__name">{printer.name}</h2>
      <span class="detail__model">{brandLabel(printer.brand)} · {printer.model}</span>
    </div>
    <button class="detail__close" on:click={() => dispatch('close')} aria-label="Закрыть">✕</button>
  </div>

  <!-- Status row -->
  <div class="detail__status-row">
    <StatusBadge status={printer.status} />
    <span class="detail__lastseen">Обновлено {formatRelativeTime(printer.lastSeen)}</span>
  </div>

  <!-- Meta info -->
  <Card padding="sm">
    <div class="detail__meta-grid">
      <div class="detail__meta-item">
        <span class="detail__meta-label">IP-адрес</span>
        <span class="detail__meta-val detail__meta-val--mono">{printer.ip}</span>
      </div>
      {#if printer.location}
        <div class="detail__meta-item">
          <span class="detail__meta-label">Расположение</span>
          <span class="detail__meta-val">{printer.location}</span>
        </div>
      {/if}
      {#if printer.group}
        <div class="detail__meta-item">
          <span class="detail__meta-label">Группа</span>
          <span class="detail__meta-val">{printer.group}</span>
        </div>
      {/if}
      {#if printer.pageCount != null}
        <div class="detail__meta-item">
          <span class="detail__meta-label">Страниц напечатано</span>
          <span class="detail__meta-val detail__meta-val--mono">
            {formatPageCount(printer.pageCount)}
          </span>
        </div>
      {/if}
      <div class="detail__meta-item">
        <span class="detail__meta-label">Добавлен</span>
        <span class="detail__meta-val">{printer.addedManually ? 'Вручную' : 'Автосканирование'}</span>
      </div>
    </div>
  </Card>

  <!-- Supplies -->
  <div class="detail__section">
    <h3 class="detail__section-title">Расходники</h3>
    {#if printer.supplies.length === 0}
      <p class="detail__empty">Нет данных — выполните опрос</p>
    {:else}
      <div class="detail__supplies">
        {#each printer.supplies as supply (supply.type)}
          <div class="detail__supply-row">
            <TonerGauge {supply} />
            <span class="detail__supply-raw">
              {supply.level} / {supply.maxLevel}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Print Spooler (Windows only) -->
  {#if isWindows}
    <div class="detail__section">
      <h3 class="detail__section-title">Print Spooler</h3>
      <div class="detail__spooler-card">
        <div class="detail__spooler-info">
          <div class="detail__spooler-icon" class:detail__spooler-icon--running={spoolerStatus === 'running'} class:detail__spooler-icon--stopped={spoolerStatus === 'stopped'} class:detail__spooler-icon--pending={spoolerStatus === 'start_pending' || spoolerStatus === 'stop_pending'}>
            <!-- Gear icon -->
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"/>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
          </div>
          <div class="detail__spooler-text">
            <span class="detail__spooler-name">Print Spooler</span>
            {#if spoolerStatus && spoolerStatus !== 'unavailable'}
              <span class="detail__spooler-status" data-status={spoolerStatus}>
                {spoolerStatusLabels[spoolerStatus] ?? spoolerStatus}
              </span>
            {/if}
          </div>
        </div>
        <button
          class="detail__spooler-btn"
          class:detail__spooler-btn--loading={restartingSpooler}
          disabled={restartingSpooler}
          on:click={restartSpooler}
          title="Перезапустить Print Spooler"
        >
          {#if restartingSpooler}
            <!-- Spinner -->
            <svg class="spin" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
          {:else}
            <!-- Refresh icon -->
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="1 4 1 10 7 10"/>
              <path d="M3.51 15a9 9 0 1 0 .49-3.54"/>
            </svg>
          {/if}
          {restartingSpooler ? 'Перезапуск…' : 'Перезапустить'}
        </button>
      </div>
    </div>
  {/if}

  <!-- Actions -->
  <div class="detail__actions">
    <Button
      variant="primary"
      size="sm"
      loading={polling}
      on:click={pollNow}
      fullWidth
    >
      {polling ? 'Опрашиваю...' : '↺ Опросить сейчас'}
    </Button>
    <Button
      variant="ghost"
      size="sm"
      on:click={() => window.location.href = `/history`}
      fullWidth
    >
      ◷ История тонера
    </Button>
    <Button
      variant="danger"
      size="sm"
      loading={removing}
      on:click={removePrinter}
      fullWidth
    >
      {removing ? 'Удаляю...' : '✕ Удалить принтер'}
    </Button>
  </div>
</aside>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .detail {
    width: 300px;
    min-width: 300px;
    background: var(--surface-1);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: v.$space-4;
    padding: v.$space-5;
    overflow-y: auto;
    @include m.custom-scrollbar;

    @include m.respond-below('lg') {
      position: fixed;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: v.$z-modal;
      box-shadow: v.$shadow-xl;
    }

    // ── Header ──
    &__header {
      @include m.flex-start;
      gap: v.$space-3;
    }

    &__brand-badge {
      width: 40px;
      height: 40px;
      border-radius: v.$radius-md;
      @include m.flex-center;
      font-family: v.$font-display;
      font-size: v.$font-size-lg;
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
      display: flex;
      flex-direction: column;
      gap: 2px;
    }

    &__name {
      font-size: v.$font-size-base;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
      @include m.truncate;
      letter-spacing: -0.01em;
    }

    &__model {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__close {
      width: 28px;
      height: 28px;
      @include m.flex-center;
      border-radius: v.$radius-md;
      color: var(--text-tertiary);
      font-size: v.$font-size-xs;
      flex-shrink: 0;
      transition: background v.$transition-fast, color v.$transition-fast;
      @include m.focus-ring;

      &:hover {
        background: var(--nav-hover-bg);
        color: var(--text-primary);
      }
    }

    // ── Status ──
    &__status-row {
      @include m.flex-between;
      gap: v.$space-2;
    }

    &__lastseen {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    // ── Meta grid ──
    &__meta-grid {
      display: flex;
      flex-direction: column;
      gap: v.$space-2;
    }

    &__meta-item {
      @include m.flex-between;
      gap: v.$space-3;
    }

    &__meta-label {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      white-space: nowrap;
    }

    &__meta-val {
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-medium;
      color: var(--text-primary);
      text-align: right;
      @include m.truncate;

      &--mono {
        font-family: v.$font-mono;
      }
    }

    // ── Section ──
    &__section {
      display: flex;
      flex-direction: column;
      gap: v.$space-3;
    }

    &__section-title {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-semibold;
      letter-spacing: v.$letter-spacing-wider;
      text-transform: uppercase;
      color: var(--text-tertiary);
    }

    &__supplies {
      display: flex;
      flex-direction: column;
      gap: v.$space-3;
    }

    &__supply-row {
      display: flex;
      flex-direction: column;
      gap: v.$space-1;
    }

    &__supply-raw {
      font-family: v.$font-mono;
      font-size: 9px;
      color: var(--text-tertiary);
      text-align: right;
    }

    &__empty {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      text-align: center;
      padding: v.$space-4;
      background: var(--surface-2);
      border-radius: v.$radius-md;
      border: 1px dashed var(--border);
    }

    // ── Spooler block ──
    &__spooler-card {
      @include m.flex-between;
      gap: v.$space-3;
      padding: v.$space-3;
      background: var(--surface-2);
      border-radius: v.$radius-md;
      border: 1px solid var(--border);
    }

    &__spooler-info {
      @include m.flex-start;
      gap: v.$space-2;
      min-width: 0;
    }

    &__spooler-icon {
      width: 28px;
      height: 28px;
      border-radius: v.$radius-sm;
      @include m.flex-center;
      flex-shrink: 0;
      color: var(--text-tertiary);
      background: var(--surface-3);
      transition: color v.$transition-base, background v.$transition-base;

      &--running {
        color: var(--status-online);
        background: rgba(34,197,94,0.12);
      }

      &--stopped {
        color: var(--status-error);
        background: rgba(239,68,68,0.10);
      }

      &--pending {
        color: var(--status-warning);
        background: rgba(245,158,11,0.10);
      }
    }

    &__spooler-text {
      display: flex;
      flex-direction: column;
      gap: 2px;
      min-width: 0;
    }

    &__spooler-name {
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-medium;
      color: var(--text-primary);
    }

    &__spooler-status {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);

      &[data-status='running']       { color: var(--status-online); }
      &[data-status='stopped']       { color: var(--status-error); }
      &[data-status='start_pending'],
      &[data-status='stop_pending']  { color: var(--status-warning); }
    }

    &__spooler-btn {
      @include m.flex-center;
      gap: v.$space-1;
      flex-shrink: 0;
      height: 28px;
      padding: 0 v.$space-3;
      border-radius: v.$radius-sm;
      font-family: v.$font-mono;
      font-size: 10px;
      font-weight: v.$font-weight-semibold;
      color: var(--accent);
      background: var(--accent-muted);
      border: 1px solid transparent;
      cursor: pointer;
      transition:
        background v.$transition-base,
        border-color v.$transition-base,
        opacity v.$transition-base;
      @include m.focus-ring;

      &:hover:not(:disabled) {
        background: rgba(0,212,170,0.20);
        border-color: rgba(0,212,170,0.3);
      }

      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
      }

      &--loading {
        pointer-events: none;
      }
    }

    // ── Actions ──
    &__actions {
      display: flex;
      flex-direction: column;
      gap: v.$space-2;
      margin-top: auto;
      padding-top: v.$space-4;
      border-top: 1px solid var(--border);
    }
  }

  // Анимация спиннера
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .spin {
    animation: spin 0.8s linear infinite;
    display: block;
  }
</style>
