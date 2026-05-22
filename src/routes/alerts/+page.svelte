<!-- src/routes/alerts/+page.svelte -->

<script lang="ts">
  import { onMount } from 'svelte';
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { printers } from '$lib/stores/printers';
  import { notifications } from '$lib/stores/notifications';
  import { api } from '$lib/api';
  import type { AlertRuleRecord } from '$lib/api';
  import { supplyLabel, statusLabel } from '$lib/utils/formatters';
  import { tonerColor } from '$lib/utils/colors';
  import type { SupplyType } from '$lib/types/printer';

  // ── Текущее состояние принтеров ───────────────────────────────────────────
  $: criticalItems = $printers.flatMap(p =>
    p.supplies.filter(s => s.isCritical).map(s => ({ printer: p, supply: s }))
  );
  $: lowItems = $printers.flatMap(p =>
    p.supplies.filter(s => s.isLow && !s.isCritical).map(s => ({ printer: p, supply: s }))
  );
  $: offlineItems = $printers.filter(p => p.status === 'offline' || p.status === 'error');

  // ── Правила алертов ───────────────────────────────────────────────────────
  let rules: AlertRuleRecord[] = [];
  let loading = true;

  // Модалка добавления/редактирования
  let showModal = false;
  let editingRule: AlertRuleRecord | null = null;

  // Форма
  let formPrinterId  = 'all';
  let formSupplyType = 'any';
  let formThreshold  = 20;
  let formEnabled    = true;
  let formDesktop    = true;
  let saving         = false;
  let deletingId: string | null = null;

  const SUPPLY_OPTIONS: Array<{ value: string; label: string }> = [
    { value: 'any',           label: 'Любой расходник' },
    { value: 'toner_black',   label: 'Тонер чёрный' },
    { value: 'toner_cyan',    label: 'Тонер голубой' },
    { value: 'toner_magenta', label: 'Тонер пурпурный' },
    { value: 'toner_yellow',  label: 'Тонер жёлтый' },
    { value: 'drum',          label: 'Барабан' },
    { value: 'fuser',         label: 'Термоузел' },
    { value: 'waste',         label: 'Контейнер отходов' },
  ];

  onMount(() => {
    loadRules();
  });

  function loadRules() {
    loading = true;
    api.getAlertRules()
      .then(r => { rules = r; })
      .catch(e => notifications.error(`Ошибка загрузки правил: ${e}`))
      .finally(() => { loading = false; });
  }

  function openAdd() {
    editingRule     = null;
    formPrinterId   = 'all';
    formSupplyType  = 'any';
    formThreshold   = 20;
    formEnabled     = true;
    formDesktop     = true;
    showModal       = true;
  }

  function openEdit(rule: AlertRuleRecord) {
    editingRule    = rule;
    formPrinterId  = rule.printerId;
    formSupplyType = rule.supplyType;
    formThreshold  = rule.threshold;
    formEnabled    = rule.enabled;
    formDesktop    = rule.notifyDesktop;
    showModal      = true;
  }

  async function saveRule() {
    saving = true;
    const rule: AlertRuleRecord = {
      id:            editingRule?.id ?? crypto.randomUUID(),
      printerId:     formPrinterId,
      supplyType:    formSupplyType,
      threshold:     formThreshold,
      enabled:       formEnabled,
      notifyDesktop: formDesktop,
    };
    try {
      await api.saveAlertRule(rule);
      notifications.success(editingRule ? 'Правило обновлено' : 'Правило добавлено');
      showModal = false;
      loadRules();
    } catch (e) {
      notifications.error(`Ошибка сохранения: ${e}`);
    } finally {
      saving = false;
    }
  }

  async function toggleEnabled(rule: AlertRuleRecord) {
    const updated = { ...rule, enabled: !rule.enabled };
    try {
      await api.saveAlertRule(updated);
      rules = rules.map(r => r.id === rule.id ? updated : r);
    } catch (e) {
      notifications.error(`Ошибка: ${e}`);
    }
  }

  async function deleteRule(id: string) {
    deletingId = id;
    try {
      await api.deleteAlertRule(id);
      rules = rules.filter(r => r.id !== id);
      notifications.success('Правило удалено');
    } catch (e) {
      notifications.error(`Ошибка удаления: ${e}`);
    } finally {
      deletingId = null;
    }
  }

  function printerName(id: string): string {
    if (id === 'all') return 'Все принтеры';
    return $printers.find(p => p.id === id)?.name ?? id;
  }

  function supplyTypeDot(type: string): string {
    const map: Record<string, string> = {
      toner_black:   'var(--text-primary)',
      toner_cyan:    '#06b6d4',
      toner_magenta: '#ec4899',
      toner_yellow:  '#eab308',
      drum:          '#8b5cf6',
      fuser:         '#f97316',
      waste:         'var(--text-tertiary)',
      any:           'var(--accent)',
    };
    return map[type] ?? 'var(--text-secondary)';
  }
</script>

<Header />

<PageWrapper>
  <div class="alerts-layout">

    <!-- ── Секция текущего состояния ─────────────────────────────────────── -->
    {#if criticalItems.length > 0 || lowItems.length > 0 || offlineItems.length > 0}
      <section class="state-section">
        <div class="section-label">Текущее состояние</div>

        {#if criticalItems.length > 0}
          <div class="alert-group">
            <div class="alert-group__header alert-group__header--critical">
              <span class="alert-group__icon">!</span>
              <span>Критические — {criticalItems.length}</span>
              <span class="alert-group__hint">Требуется немедленная замена</span>
            </div>
            {#each criticalItems as { printer, supply }}
              <div class="state-row state-row--critical animate-fade-in-up">
                <div class="state-row__info">
                  <span class="state-row__name">{printer.name}</span>
                  <span class="state-row__sub">{supplyLabel(supply.type as SupplyType)}</span>
                  <span class="state-row__ip">{printer.ip}</span>
                </div>
                <div class="state-row__gauge">
                  <span class="state-row__pct" style="color:{tonerColor(supply.percent)}">{supply.percent}%</span>
                  <div class="mini-bar"><div class="mini-bar__fill" style="width:{supply.percent}%;background:{tonerColor(supply.percent)}"></div></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if lowItems.length > 0}
          <div class="alert-group">
            <div class="alert-group__header alert-group__header--warning">
              <span class="alert-group__icon">↓</span>
              <span>Заканчивается — {lowItems.length}</span>
              <span class="alert-group__hint">Ниже порогового значения</span>
            </div>
            {#each lowItems as { printer, supply }}
              <div class="state-row state-row--warning animate-fade-in-up">
                <div class="state-row__info">
                  <span class="state-row__name">{printer.name}</span>
                  <span class="state-row__sub">{supplyLabel(supply.type as SupplyType)}</span>
                  <span class="state-row__ip">{printer.ip}</span>
                </div>
                <div class="state-row__gauge">
                  <span class="state-row__pct" style="color:{tonerColor(supply.percent)}">{supply.percent}%</span>
                  <div class="mini-bar"><div class="mini-bar__fill" style="width:{supply.percent}%;background:{tonerColor(supply.percent)}"></div></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if offlineItems.length > 0}
          <div class="alert-group">
            <div class="alert-group__header alert-group__header--offline">
              <span class="alert-group__icon">◌</span>
              <span>Недоступны — {offlineItems.length}</span>
              <span class="alert-group__hint">Не отвечают или в состоянии ошибки</span>
            </div>
            {#each offlineItems as printer}
              <div class="state-row state-row--offline animate-fade-in-up">
                <div class="state-row__info">
                  <span class="state-row__name">{printer.name}</span>
                  <span class="state-row__ip">{printer.ip}</span>
                </div>
                <Badge variant={printer.status === 'error' ? 'error' : 'neutral'}>
                  {statusLabel(printer.status)}
                </Badge>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {:else}
      <div class="all-clear">
        <div class="all-clear__icon">✓</div>
        <h2 class="all-clear__title">Всё в порядке</h2>
        <p class="all-clear__desc">Все принтеры работают нормально, расходники в норме</p>
      </div>
    {/if}

    <!-- ── Секция правил алертов ──────────────────────────────────────────── -->
    <section class="rules-section">
      <div class="rules-header">
        <div>
          <div class="section-label">Правила уведомлений</div>
          <p class="rules-header__hint">
            При совпадении условий шедулер отправит desktop-уведомление
          </p>
        </div>
        <Button variant="primary" on:click={openAdd}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" style="flex-shrink:0">
            <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
          </svg>
          Добавить правило
        </Button>
      </div>

      {#if loading}
        <div class="rules-skeleton">
          {#each [1, 2, 3] as _}
            <div class="skeleton-row">
              <div class="skeleton skeleton--md"></div>
              <div class="skeleton skeleton--sm"></div>
              <div class="skeleton skeleton--xs"></div>
            </div>
          {/each}
        </div>
      {:else if rules.length === 0}
        <div class="rules-empty">
          <div class="rules-empty__icon">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
              <path d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" stroke="var(--text-tertiary)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <p class="rules-empty__text">Нет правил уведомлений</p>
          <p class="rules-empty__hint">Создайте правило, чтобы получать desktop-уведомления когда тонер заканчивается</p>
          <Button variant="outline" on:click={openAdd}>Создать первое правило</Button>
        </div>
      {:else}
        <div class="rules-table">
          <!-- Заголовок таблицы -->
          <div class="rules-table__head">
            <span>Принтер</span>
            <span>Расходник</span>
            <span>Порог</span>
            <span>Desktop</span>
            <span>Активно</span>
            <span></span>
          </div>

          {#each rules as rule (rule.id)}
            <div class="rules-table__row" class:rules-table__row--disabled={!rule.enabled}>
              <!-- Принтер -->
              <div class="rule-cell rule-cell--printer">
                {#if rule.printerId === 'all'}
                  <span class="rule-all-badge">Все</span>
                {:else}
                  <span class="rule-printer-name">{printerName(rule.printerId)}</span>
                {/if}
              </div>

              <!-- Расходник -->
              <div class="rule-cell rule-cell--supply">
                <span class="supply-dot" style="background:{supplyTypeDot(rule.supplyType)}"></span>
                <span>
                  {rule.supplyType === 'any'
                    ? 'Любой'
                    : supplyLabel(rule.supplyType as SupplyType)}
                </span>
              </div>

              <!-- Порог -->
              <div class="rule-cell">
                <span class="threshold-badge" class:threshold-badge--crit={rule.threshold <= 10} class:threshold-badge--low={rule.threshold > 10 && rule.threshold <= 20}>
                  ≤ {rule.threshold}%
                </span>
              </div>

              <!-- Desktop toggle -->
              <div class="rule-cell">
                <button
                  class="icon-toggle"
                  class:icon-toggle--on={rule.notifyDesktop}
                  title={rule.notifyDesktop ? 'Desktop-уведомления включены' : 'Desktop-уведомления выключены'}
                  on:click={() => api.saveAlertRule({ ...rule, notifyDesktop: !rule.notifyDesktop }).then(loadRules)}
                  aria-label="Переключить desktop-уведомление"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </div>

              <!-- Enabled toggle -->
              <div class="rule-cell">
                <button
                  class="toggle-switch"
                  class:toggle-switch--on={rule.enabled}
                  on:click={() => toggleEnabled(rule)}
                  aria-label={rule.enabled ? 'Отключить правило' : 'Включить правило'}
                  title={rule.enabled ? 'Правило активно' : 'Правило отключено'}
                >
                  <span class="toggle-switch__knob"></span>
                </button>
              </div>

              <!-- Actions -->
              <div class="rule-cell rule-cell--actions">
                <button class="action-btn action-btn--edit" on:click={() => openEdit(rule)} title="Редактировать">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
                    <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
                <button
                  class="action-btn action-btn--delete"
                  on:click={() => deleteRule(rule.id)}
                  disabled={deletingId === rule.id}
                  title="Удалить"
                >
                  {#if deletingId === rule.id}
                    <span class="spin">⟳</span>
                  {:else}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
                      <path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                  {/if}
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

  </div>
</PageWrapper>

<!-- ── Модалка добавления/редактирования ──────────────────────────────────── -->
<Modal
  open={showModal}
  title={editingRule ? 'Редактировать правило' : 'Новое правило алерта'}
  size="sm"
  on:close={() => (showModal = false)}
>
  <div class="rule-form">

    <!-- Принтер -->
    <div class="form-field">
      <label class="form-label" for="rule-printer">Принтер</label>
      <select id="rule-printer" class="form-select" bind:value={formPrinterId}>
        <option value="all">Все принтеры</option>
        {#each $printers as p}
          <option value={p.id}>{p.name} ({p.ip})</option>
        {/each}
      </select>
    </div>

    <!-- Тип расходника -->
    <div class="form-field">
      <label class="form-label" for="rule-supply">Расходник</label>
      <select id="rule-supply" class="form-select" bind:value={formSupplyType}>
        {#each SUPPLY_OPTIONS as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <!-- Порог -->
    <div class="form-field">
      <label class="form-label" for="rule-threshold">
        Порог срабатывания
        <span class="form-label__value" style="color:{tonerColor(formThreshold)}">{formThreshold}%</span>
      </label>
      <div class="threshold-slider-wrap">
        <input
          id="rule-threshold"
          type="range"
          min="1"
          max="99"
          step="1"
          bind:value={formThreshold}
          class="threshold-slider"
          style="--pct:{formThreshold}%"
        />
        <div class="threshold-marks">
          <span class:threshold-marks__mark--active={formThreshold <= 10} style="color:var(--status-error)">Крит ≤10%</span>
          <span class:threshold-marks__mark--active={formThreshold > 10 && formThreshold <= 20} style="color:var(--status-warning)">Низко ≤20%</span>
          <span class:threshold-marks__mark--active={formThreshold > 20}>Норма</span>
        </div>
      </div>
    </div>

    <!-- Чекбоксы -->
    <div class="form-checks">
      <label class="check-row">
        <input type="checkbox" bind:checked={formDesktop} class="check-input" />
        <span class="check-label">
          Desktop-уведомления
          <span class="check-hint">Нативное уведомление ОС при срабатывании</span>
        </span>
      </label>
      <label class="check-row">
        <input type="checkbox" bind:checked={formEnabled} class="check-input" />
        <span class="check-label">
          Правило активно
          <span class="check-hint">Снимите галочку чтобы временно отключить</span>
        </span>
      </label>
    </div>

  </div>

  <svelte:fragment slot="footer">
    <Button variant="ghost" on:click={() => (showModal = false)}>Отмена</Button>
    <Button variant="primary" loading={saving} on:click={saveRule}>
      {editingRule ? 'Сохранить' : 'Создать'}
    </Button>
  </svelte:fragment>
</Modal>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .alerts-layout {
    display: flex;
    flex-direction: column;
    gap: v.$space-6;
  }

  // ── Заголовок секции ──────────────────────────────────────────────────────

  .section-label {
    font-size: v.$font-size-xs;
    font-weight: v.$font-weight-semibold;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
    margin-bottom: v.$space-3;
  }

  // ── Текущее состояние ─────────────────────────────────────────────────────

  .state-section {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;
  }

  .alert-group {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;

    &__header {
      @include m.flex-start;
      gap: v.$space-2;
      padding: v.$space-2 v.$space-3;
      border-radius: v.$radius-sm;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-semibold;
      border: 1px solid;

      &--critical {
        background: rgba(239,68,68,0.06);
        border-color: rgba(239,68,68,0.2);
        color: var(--status-error);
      }
      &--warning {
        background: rgba(245,158,11,0.06);
        border-color: rgba(245,158,11,0.2);
        color: var(--status-warning);
      }
      &--offline {
        background: rgba(113,113,122,0.06);
        border-color: var(--border);
        color: var(--text-secondary);
      }
    }

    &__icon { flex-shrink: 0; }

    &__hint {
      color: var(--text-tertiary);
      margin-left: auto;
    }
  }

  .state-row {
    @include m.flex-between;
    gap: v.$space-4;
    padding: v.$space-2 v.$space-3;
    border-radius: v.$radius-md;
    border: 1px solid;

    &--critical {
      background: rgba(239,68,68,0.03);
      border-color: rgba(239,68,68,0.12);
    }
    &--warning {
      background: rgba(245,158,11,0.03);
      border-color: rgba(245,158,11,0.12);
    }
    &--offline {
      background: var(--surface-1);
      border-color: var(--border);
    }

    &__info {
      @include m.flex-start;
      gap: v.$space-3;
      min-width: 0;
    }

    &__name {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-medium;
      color: var(--text-primary);
      @include m.truncate;
    }

    &__sub {
      font-size: v.$font-size-xs;
      color: var(--text-secondary);
    }

    &__ip {
      font-family: v.$font-mono;
      font-size: 10px;
      color: var(--text-tertiary);
    }

    &__gauge {
      @include m.flex-start;
      gap: v.$space-2;
      flex-shrink: 0;
    }

    &__pct {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-bold;
      min-width: 36px;
      text-align: right;
    }
  }

  .mini-bar {
    width: 60px;
    height: 3px;
    background: var(--gauge-track);
    border-radius: v.$radius-full;
    overflow: hidden;

    &__fill {
      height: 100%;
      border-radius: v.$radius-full;
      transition: width 0.4s ease;
    }
  }

  // ── All clear ─────────────────────────────────────────────────────────────

  .all-clear {
    @include m.flex-center;
    flex-direction: column;
    gap: v.$space-4;
    padding: v.$space-8;
    text-align: center;

    &__icon {
      width: 56px;
      height: 56px;
      border-radius: 50%;
      background: rgba(34,197,94,0.10);
      border: 2px solid rgba(34,197,94,0.25);
      @include m.flex-center;
      font-size: 24px;
      color: var(--status-online);
    }

    &__title {
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }

    &__desc {
      font-size: v.$font-size-sm;
      color: var(--text-tertiary);
    }
  }

  // ── Правила алертов ───────────────────────────────────────────────────────

  .rules-section {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;
  }

  .rules-header {
    @include m.flex-between;
    gap: v.$space-4;

    &__hint {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      margin-top: 2px;
    }
  }

  // Skeleton
  .rules-skeleton {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
  }

  .skeleton-row {
    @include m.flex-start;
    gap: v.$space-4;
    padding: v.$space-3 v.$space-4;
    border-radius: v.$radius-md;
    background: var(--surface-1);
    border: 1px solid var(--border);
  }

  .skeleton {
    height: 12px;
    border-radius: v.$radius-sm;
    background: var(--surface-3);
    animation: shimmer 1.4s ease-in-out infinite;

    &--md  { width: 160px; }
    &--sm  { width: 100px; }
    &--xs  { width: 48px; }
  }

  // Empty state
  .rules-empty {
    @include m.flex-center;
    flex-direction: column;
    gap: v.$space-3;
    padding: v.$space-8 v.$space-4;
    border-radius: v.$radius-lg;
    border: 1px dashed var(--border-hover);
    text-align: center;

    &__icon {
      width: 52px;
      height: 52px;
      border-radius: 50%;
      background: var(--surface-2);
      @include m.flex-center;
    }

    &__text {
      font-size: v.$font-size-base;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
    }

    &__hint {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      max-width: 320px;
    }
  }

  // Таблица правил
  .rules-table {
    @include m.card-base;
    overflow: hidden;

    &__head {
      display: grid;
      grid-template-columns: 1fr 1fr 90px 52px 64px 72px;
      gap: v.$space-3;
      padding: v.$space-2 v.$space-4;
      background: var(--surface-2);
      border-bottom: 1px solid var(--border);
      font-size: 10px;
      font-weight: v.$font-weight-semibold;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--text-tertiary);
    }

    &__row {
      display: grid;
      grid-template-columns: 1fr 1fr 90px 52px 64px 72px;
      gap: v.$space-3;
      align-items: center;
      padding: v.$space-3 v.$space-4;
      border-bottom: 1px solid var(--border);
      transition: background v.$transition-fast;

      &:last-child { border-bottom: none; }
      &:hover { background: var(--surface-2); }

      &--disabled {
        opacity: 0.5;
      }
    }
  }

  .rule-cell {
    @include m.flex-start;
    gap: v.$space-2;
    min-width: 0;
    font-size: v.$font-size-sm;
    color: var(--text-primary);

    &--printer { font-weight: v.$font-weight-medium; }

    &--supply {
      color: var(--text-secondary);
    }

    &--actions {
      @include m.flex-start;
      gap: v.$space-1;
      justify-content: flex-end;
    }
  }

  .rule-all-badge {
    font-size: 10px;
    font-weight: v.$font-weight-semibold;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 2px 7px;
    border-radius: v.$radius-sm;
    background: var(--accent-muted);
    color: var(--accent);
  }

  .rule-printer-name {
    @include m.truncate;
  }

  .supply-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .threshold-badge {
    font-family: v.$font-mono;
    font-size: v.$font-size-xs;
    font-weight: v.$font-weight-bold;
    padding: 2px 8px;
    border-radius: v.$radius-sm;
    background: var(--surface-3);
    color: var(--text-secondary);

    &--crit {
      background: rgba(239,68,68,0.12);
      color: var(--status-error);
    }

    &--low {
      background: rgba(245,158,11,0.12);
      color: var(--status-warning);
    }
  }

  // Toggle switch
  .toggle-switch {
    position: relative;
    width: 34px;
    height: 18px;
    border-radius: 9px;
    background: var(--surface-3);
    border: none;
    cursor: pointer;
    transition: background v.$transition-base;
    flex-shrink: 0;

    &--on { background: var(--accent); }

    &__knob {
      position: absolute;
      top: 2px;
      left: 2px;
      width: 14px;
      height: 14px;
      border-radius: 50%;
      background: white;
      transition: transform v.$transition-base;
      pointer-events: none;
    }

    &--on &__knob {
      transform: translateX(16px);
    }
  }

  // Icon toggle (desktop bell)
  .icon-toggle {
    width: 28px;
    height: 28px;
    border-radius: v.$radius-sm;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    @include m.flex-center;
    transition: all v.$transition-fast;

    &--on {
      border-color: var(--accent);
      color: var(--accent);
      background: var(--accent-muted);
    }

    &:hover { border-color: var(--border-hover); color: var(--text-primary); }
    &--on:hover { border-color: var(--accent); }
  }

  // Action buttons
  .action-btn {
    width: 28px;
    height: 28px;
    border-radius: v.$radius-sm;
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    @include m.flex-center;
    transition: all v.$transition-fast;
    color: var(--text-tertiary);

    &--edit:hover {
      border-color: var(--border-hover);
      color: var(--text-primary);
      background: var(--surface-2);
    }

    &--delete:hover {
      border-color: rgba(239,68,68,0.3);
      color: var(--status-error);
      background: rgba(239,68,68,0.06);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  }

  .spin {
    display: inline-block;
    animation: spin 0.8s linear infinite;
  }

  // ── Форма в модалке ────────────────────────────────────────────────────────

  .rule-form {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
  }

  .form-label {
    @include m.flex-between;
    font-size: v.$font-size-xs;
    font-weight: v.$font-weight-semibold;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;

    &__value {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-black;
      text-transform: none;
      letter-spacing: 0;
    }
  }

  .form-select {
    width: 100%;
    padding: v.$space-2 v.$space-3;
    border-radius: v.$radius-md;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-primary);
    font-size: v.$font-size-sm;
    font-family: v.$font-display;
    outline: none;
    cursor: pointer;
    transition: border-color v.$transition-fast;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%238b95a1' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 12px center;
    padding-right: 32px;

    &:focus { border-color: var(--accent); }

    option {
      background: var(--surface-2);
      color: var(--text-primary);
    }
  }

  // Слайдер порога
  .threshold-slider-wrap {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
  }

  .threshold-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: v.$radius-full;
    background: linear-gradient(
      to right,
      var(--accent) 0%,
      var(--accent) var(--pct),
      var(--surface-3) var(--pct),
      var(--surface-3) 100%
    );
    outline: none;
    cursor: pointer;

    &::-webkit-slider-thumb {
      -webkit-appearance: none;
      width: 16px;
      height: 16px;
      border-radius: 50%;
      background: var(--accent);
      border: 2px solid var(--bg);
      box-shadow: 0 0 0 2px var(--accent);
      cursor: pointer;
      transition: box-shadow v.$transition-fast;
    }

    &:hover::-webkit-slider-thumb {
      box-shadow: 0 0 0 4px var(--accent-muted);
    }
  }

  .threshold-marks {
    @include m.flex-between;
    font-size: 10px;
    color: var(--text-tertiary);

    &__mark--active {
      font-weight: v.$font-weight-semibold;
    }
  }

  // Чекбоксы
  .form-checks {
    display: flex;
    flex-direction: column;
    gap: v.$space-3;
    padding: v.$space-3;
    border-radius: v.$radius-md;
    background: var(--surface-2);
    border: 1px solid var(--border);
  }

  .check-row {
    @include m.flex-start;
    gap: v.$space-3;
    cursor: pointer;
  }

  .check-input {
    width: 15px;
    height: 15px;
    accent-color: var(--accent);
    cursor: pointer;
    flex-shrink: 0;
  }

  .check-label {
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-size: v.$font-size-sm;
    color: var(--text-primary);
  }

  .check-hint {
    font-size: v.$font-size-xs;
    color: var(--text-tertiary);
  }
</style>
