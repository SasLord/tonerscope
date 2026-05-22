<!-- src/lib/components/ui/Input.svelte -->

<script lang="ts">
  export let value    = '';
  export let type:    'text' | 'password' | 'number' | 'email' | 'search' = 'text';
  export let placeholder = '';
  export let label    = '';
  export let hint     = '';
  export let error    = '';
  export let disabled = false;
  export let id       = crypto.randomUUID();

  // icon slot support
</script>

<div class="field" class:field--error={!!error} class:field--disabled={disabled}>
  {#if label}
    <label class="field__label" for={id}>{label}</label>
  {/if}
  <div class="field__wrap">
    {#if $$slots.prefix}
      <span class="field__prefix"><slot name="prefix" /></span>
    {/if}
    <input
      {id} {type} {placeholder} {disabled}
      class="field__input"
      class:field__input--prefixed={$$slots.prefix}
      class:field__input--suffixed={$$slots.suffix}
      bind:value
      on:input
      on:change
      on:blur
      on:focus
      on:keydown
      {...$$restProps}
    />
    {#if $$slots.suffix}
      <span class="field__suffix"><slot name="suffix" /></span>
    {/if}
  </div>
  {#if error}
    <span class="field__hint field__hint--error">{error}</span>
  {:else if hint}
    <span class="field__hint">{hint}</span>
  {/if}
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .field {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;

    &__label {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-medium;
      color: var(--text-secondary);
      letter-spacing: 0.01em;
    }

    &__wrap {
      position: relative;
      display: flex;
      align-items: center;
    }

    &__input {
      width: 100%;
      height: 36px;
      padding: 0 v.$space-3;
      background: var(--surface-1);
      border: 1px solid var(--border);
      border-radius: v.$radius-md;
      color: var(--text-primary);
      font-size: v.$font-size-base;
      font-family: v.$font-body;
      transition: border-color v.$transition-fast, box-shadow v.$transition-fast;
      @include m.focus-ring(var(--accent));

      &::placeholder { color: var(--text-tertiary); }

      &:focus {
        outline: none;
        border-color: var(--accent);
        box-shadow: 0 0 0 3px var(--accent-muted);
      }

      &:hover:not(:focus) { border-color: var(--border-hover); }

      &--prefixed { padding-left: 36px; }
      &--suffixed { padding-right: 36px; }
    }

    &__prefix, &__suffix {
      position: absolute;
      top: 50%;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      color: var(--text-tertiary);
      pointer-events: none;
    }
    &__prefix { left: v.$space-3; }
    &__suffix { right: v.$space-3; }

    &__hint {
      font-size: v.$font-size-xs;
      color: var(--text-tertiary);
      &--error { color: var(--status-error); }
    }

    &--error .field__input {
      border-color: var(--status-error);
      &:focus { box-shadow: 0 0 0 3px rgba(239,68,68,0.12); }
    }

    &--disabled {
      opacity: 0.5;
      pointer-events: none;
    }
  }
</style>
