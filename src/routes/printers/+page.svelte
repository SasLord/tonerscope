<!-- src/routes/printers/+page.svelte -->

<script lang="ts">
  import Header from '$lib/components/layout/Header.svelte';
  import PageWrapper from '$lib/components/layout/PageWrapper.svelte';
  import PrinterGrid from '$lib/components/printer/PrinterGrid.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { printers } from '$lib/stores/printers';
  import { notifications } from '$lib/stores/notifications';
  import type { PrinterBrand, PrinterInfo } from '$lib/types/printer';

  let searchValue = '';
  let filterStatus: string = 'all';
  let filterBrand:  string = 'all';
  let addModalOpen = false;

  // New printer form
  let newIp    = '';
  let newName  = '';
  let newBrand: PrinterBrand = 'pantum';
  let newLoc   = '';

  const statusFilters = [
    { value: 'all',      label: 'Все'         },
    { value: 'online',   label: 'В сети'      },
    { value: 'offline',  label: 'Не в сети'   },
    { value: 'error',    label: 'Ошибка'      },
    { value: 'warning',  label: 'Предупрежд.' },
  ];

  const brandFilters = [
    { value: 'all',     label: 'Все бренды' },
    { value: 'pantum',  label: 'Pantum'     },
    { value: 'kyocera', label: 'Kyocera'    },
    { value: 'hp',      label: 'HP'         },
    { value: 'canon',   label: 'Canon'      },
    { value: 'other',   label: 'Другой'     },
  ];

  $: filtered = $printers.filter(p => {
    const matchSearch = !searchValue ||
      p.name.toLowerCase().includes(searchValue.toLowerCase()) ||
      p.ip.includes(searchValue) ||
      p.model.toLowerCase().includes(searchValue.toLowerCase());
    const matchStatus = filterStatus === 'all' || p.status === filterStatus;
    const matchBrand  = filterBrand  === 'all' || p.brand  === filterBrand;
    return matchSearch && matchStatus && matchBrand;
  });

  function addPrinter() {
    if (!newIp || !newName) return;
    const printer: PrinterInfo = {
      id: crypto.randomUUID(),
      ip: newIp,
      name: newName,
      brand: newBrand,
      model: '',
      location: newLoc,
      status: 'unknown',
      supplies: [],
      lastSeen: new Date().toISOString(),
      addedManually: true,
    };
    printers.upsert(printer);
    notifications.success('Принтер добавлен', `${newName} (${newIp})`);
    addModalOpen = false;
    newIp = ''; newName = ''; newLoc = '';
  }
</script>

<Header searchable bind:searchValue>
  <svelte:fragment slot="actions">
    <Button variant="primary" size="sm" on:click={() => addModalOpen = true}>
      + Добавить
    </Button>
  </svelte:fragment>
</Header>

<PageWrapper>
  <!-- Filter bar -->
  <div class="filter-bar">
    <div class="filter-group">
      {#each statusFilters as f}
        <button
          class="filter-chip"
          class:filter-chip--active={filterStatus === f.value}
          on:click={() => filterStatus = f.value}
        >
          {f.label}
        </button>
      {/each}
    </div>

    <div class="filter-sep"></div>

    <div class="filter-group">
      {#each brandFilters as f}
        <button
          class="filter-chip"
          class:filter-chip--active={filterBrand === f.value}
          on:click={() => filterBrand = f.value}
        >
          {f.label}
        </button>
      {/each}
    </div>

    <span class="filter-count">{filtered.length} / {$printers.length}</span>
  </div>

  <PrinterGrid printers={filtered} />
</PageWrapper>

<!-- Add printer modal -->
<Modal bind:open={addModalOpen} title="Добавить принтер" size="sm">
  <div class="add-form">
    <Input label="IP-адрес *" placeholder="192.168.1.100" bind:value={newIp} />
    <Input label="Название *" placeholder="Pantum BM5100 — Бухгалтерия" bind:value={newName} />
    <div class="add-form__field">
      <label class="add-form__label" for="brand-select">Бренд</label>
      <select id="brand-select" class="add-form__select" bind:value={newBrand}>
        <option value="pantum">Pantum</option>
        <option value="kyocera">Kyocera</option>
        <option value="hp">HP</option>
        <option value="canon">Canon</option>
        <option value="other">Другой</option>
      </select>
    </div>
    <Input label="Расположение" placeholder="2 этаж, каб. 210" bind:value={newLoc} />
  </div>

  <svelte:fragment slot="footer">
    <Button variant="ghost" on:click={() => addModalOpen = false}>Отмена</Button>
    <Button variant="primary" on:click={addPrinter} disabled={!newIp || !newName}>
      Добавить
    </Button>
  </svelte:fragment>
</Modal>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .filter-bar {
    @include m.flex-start;
    gap: v.$space-3;
    flex-wrap: wrap;
    margin-bottom: v.$space-5;
  }

  .filter-group {
    @include m.flex-start;
    gap: v.$space-1;
    flex-wrap: wrap;
  }

  .filter-chip {
    height: 28px;
    padding: 0 v.$space-3;
    border-radius: v.$radius-full;
    font-size: v.$font-size-xs;
    font-weight: v.$font-weight-medium;
    color: var(--text-secondary);
    background: var(--surface-1);
    border: 1px solid var(--border);
    transition: all v.$transition-fast;
    cursor: pointer;
    @include m.focus-ring;

    &:hover {
      background: var(--surface-2);
      border-color: var(--border-hover);
      color: var(--text-primary);
    }

    &--active {
      background: var(--accent-muted);
      border-color: var(--accent);
      color: var(--accent);
      font-weight: v.$font-weight-semibold;
    }
  }

  .filter-sep {
    width: 1px;
    height: 20px;
    background: var(--border);
    flex-shrink: 0;
  }

  .filter-count {
    @include m.text-mono(v.$font-size-xs);
    color: var(--text-tertiary);
    margin-left: auto;
  }

  // Add form
  .add-form {
    display: flex;
    flex-direction: column;
    gap: v.$space-4;

    &__label {
      display: block;
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-medium;
      color: var(--text-secondary);
      margin-bottom: v.$space-2;
    }

    &__select {
      width: 100%;
      height: 36px;
      padding: 0 v.$space-3;
      background: var(--surface-1);
      border: 1px solid var(--border);
      border-radius: v.$radius-md;
      color: var(--text-primary);
      font-size: v.$font-size-base;
      cursor: pointer;
      @include m.focus-ring;

      &:focus {
        outline: none;
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-muted);
      }

      option { background: var(--surface-2); }
    }
  }
</style>
