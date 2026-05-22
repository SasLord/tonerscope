<!-- src/routes/settings/+page.svelte -->

<script lang="ts">
  import { onMount } from 'svelte';
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { settings, theme, type Theme } from '$lib/stores/settings';
  import { notifications } from '$lib/stores/notifications';
  import { api } from '$lib/api/tauri';

  let newSubnet = '';
  let saving    = false;
  let loading   = true;

  // ─── Загрузка настроек из бэкенда ────────────────────────────────────────────
  // AppSettings в Rust не содержит поля theme (не сохраняется через save_settings).
  // theme хранится в $lib/stores/settings и персистируется в localStorage.

  onMount(async () => {
    try {
      const record = await api.getSettings();
      // record — camelCase (serde rename_all), совпадает с AppSettings кроме theme
      settings.patch({
        pollIntervalMinutes:    record.pollIntervalMinutes,
        lowTonerThreshold:      record.lowTonerThreshold,
        criticalTonerThreshold: record.criticalTonerThreshold,
        snmpCommunity:          record.snmpCommunity,
        snmpTimeout:            record.snmpTimeout,
        snmpRetries:            record.snmpRetries,
        subnets:                record.subnets,
      });
    } catch (err) {
      // Бэкенд недоступен — работаем с defaults из store
      console.warn('[settings] getSettings failed, using defaults:', err);
    } finally {
      loading = false;
    }
  });

  // ─── Сохранение ───────────────────────────────────────────────────────────────

  async function saveSettings() {
    saving = true;
    try {
      // Передаём только поля, которые знает Rust (без theme)
      await api.saveSettings({
        pollIntervalMinutes:    $settings.pollIntervalMinutes,
        lowTonerThreshold:      $settings.lowTonerThreshold,
        criticalTonerThreshold: $settings.criticalTonerThreshold,
        snmpCommunity:          $settings.snmpCommunity,
        snmpTimeout:            $settings.snmpTimeout,
        snmpRetries:            $settings.snmpRetries,
        subnets:                $settings.subnets,
      });
      notifications.success('Настройки сохранены');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      notifications.error('Ошибка сохранения', msg);
    } finally {
      saving = false;
    }
  }

  function addSubnet() {
    const val = newSubnet.trim();
    if (!val) return;
    settings.patch({ subnets: [...$settings.subnets, val] });
    newSubnet = '';
  }

  function removeSubnet(s: string) {
    settings.patch({ subnets: $settings.subnets.filter(x => x !== s) });
  }

  const themeOptions: { value: Theme; label: string; icon: string }[] = [
    { value: 'dark',   label: 'Тёмная',    icon: '☾' },
    { value: 'light',  label: 'Светлая',   icon: '☀' },
    { value: 'system', label: 'Системная', icon: '⬡' },
  ];
</script>

<Header />

<PageWrapper>
  <div class="settings-layout">

    {#if loading}
      <div class="settings-loading">Загрузка настроек...</div>
    {:else}

      <!-- SNMP -->
      <Card padding="md">
        <div class="settings-section">
          <div class="settings-section__header">
            <h3 class="settings-section__title">SNMP</h3>
            <p class="settings-section__desc">Параметры подключения к принтерам</p>
          </div>
          <div class="settings-section__fields">
            <Input
              label="Community string"
              placeholder="public"
              bind:value={$settings.snmpCommunity}
            />
            <div class="settings-row">
              <Input
                label="Таймаут (сек.)"
                type="number"
                bind:value={$settings.snmpTimeout}
              />
              <Input
                label="Попыток"
                type="number"
                bind:value={$settings.snmpRetries}
              />
            </div>
            <Input
              label="Интервал опроса (мин.)"
              type="number"
              bind:value={$settings.pollIntervalMinutes}
            />
          </div>
        </div>
      </Card>

      <!-- Thresholds -->
      <Card padding="md">
        <div class="settings-section">
          <div class="settings-section__header">
            <h3 class="settings-section__title">Пороги уведомлений</h3>
            <p class="settings-section__desc">При достижении порога создаётся уведомление</p>
          </div>
          <div class="settings-section__fields">
            <div class="threshold-row">
              <div class="threshold-dot threshold-dot--warning"></div>
              <Input
                label="Предупреждение (% тонера)"
                type="number"
                bind:value={$settings.lowTonerThreshold}
              />
            </div>
            <div class="threshold-row">
              <div class="threshold-dot threshold-dot--critical"></div>
              <Input
                label="Критический (% тонера)"
                type="number"
                bind:value={$settings.criticalTonerThreshold}
              />
            </div>
          </div>
        </div>
      </Card>

      <!-- Subnets -->
      <Card padding="md">
        <div class="settings-section">
          <div class="settings-section__header">
            <h3 class="settings-section__title">Подсети для сканирования</h3>
            <p class="settings-section__desc">CIDR нотация, например 192.168.1.0/24</p>
          </div>
          <div class="settings-section__fields">
            <div class="subnet-list">
              {#each $settings.subnets as subnet}
                <div class="subnet-item">
                  <span class="subnet-item__val">{subnet}</span>
                  <button class="subnet-item__remove" on:click={() => removeSubnet(subnet)}>✕</button>
                </div>
              {:else}
                <p class="subnet-empty">Подсети не добавлены</p>
              {/each}
            </div>
            <div class="subnet-add">
              <Input
                placeholder="192.168.2.0/24"
                bind:value={newSubnet}
                on:keydown={(e) => e.key === 'Enter' && addSubnet()}
              />
              <Button variant="outline" size="sm" on:click={addSubnet} disabled={!newSubnet.trim()}>
                + Добавить
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <!-- Theme -->
      <Card padding="md">
        <div class="settings-section">
          <div class="settings-section__header">
            <h3 class="settings-section__title">Оформление</h3>
            <p class="settings-section__desc">Тема сохраняется локально, не синхронизируется с сервером</p>
          </div>
          <div class="theme-options">
            {#each themeOptions as opt}
              <button
                class="theme-btn"
                class:theme-btn--active={$theme === opt.value}
                on:click={() => theme.set(opt.value)}
              >
                <span class="theme-btn__icon">{opt.icon}</span>
                <span class="theme-btn__label">{opt.label}</span>
              </button>
            {/each}
          </div>
        </div>
      </Card>

      <div class="settings-footer">
        <Button variant="primary" loading={saving} on:click={saveSettings}>
          {saving ? 'Сохраняю...' : 'Сохранить настройки'}
        </Button>
      </div>

    {/if}
  </div>
</PageWrapper>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .settings-layout {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;
    max-width: 640px;
  }

  .settings-loading {
    font-size: v.$font-size-sm;
    color: var(--text-tertiary);
    padding: v.$space-8;
    text-align: center;
    font-family: v.$font-mono;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;

    &__header {
      display: flex;
      flex-direction: column;
      gap: 2px;
      padding-bottom: v.$space-3;
      border-bottom: 1px solid var(--border);
    }

    &__title {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
    }

    &__desc {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
    }

    &__fields {
      display: flex;
      flex-direction: column;
      gap: v.$space-3;
    }
  }

  .settings-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: v.$space-3;
  }

  // Thresholds
  .threshold-row {
    @include m.flex-start;
    gap: v.$space-3;
    align-items: flex-end;
  }

  .threshold-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-bottom: 13px;

    &--warning  { background: var(--status-warning); }
    &--critical { background: var(--status-error); }
  }

  // Subnets
  .subnet-list {
    display: flex;
    flex-direction: column;
    gap: v.$space-1;
  }

  .subnet-empty {
    font-size: v.$font-size-xs;
    color: var(--text-tertiary);
    font-family: v.$font-mono;
  }

  .subnet-item {
    @include m.flex-between;
    gap: v.$space-2;
    padding: v.$space-2 v.$space-3;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: v.$radius-md;

    &__val {
      font-family: v.$font-mono;
      font-size: v.$font-size-sm;
      color: var(--text-primary);
    }

    &__remove {
      color: var(--text-tertiary);
      font-size: v.$font-size-xs;
      padding: 2px 6px;
      border-radius: v.$radius-sm;
      transition: color v.$transition-fast;
      &:hover { color: var(--status-error); }
    }
  }

  .subnet-add {
    @include m.flex-start;
    gap: v.$space-2;
    align-items: flex-end;

    :global(.field) { flex: 1; }
  }

  // Theme
  .theme-options {
    display: flex;
    gap: v.$space-2;
  }

  .theme-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: v.$space-2;
    padding: v.$space-4 v.$space-3;
    border-radius: v.$radius-md;
    background: var(--surface-2);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all v.$transition-fast;
    @include m.focus-ring;

    &:hover {
      background: var(--surface-3);
      border-color: var(--border-hover);
    }

    &--active {
      background: var(--accent-muted);
      border-color: var(--accent);
    }

    &__icon {
      font-size: 20px;
      line-height: 1;
    }

    &__label {
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-medium;
      color: var(--text-secondary);
    }
  }

  .settings-footer {
    padding-top: v.$space-2;
    display: flex;
    justify-content: flex-end;
  }
</style>
