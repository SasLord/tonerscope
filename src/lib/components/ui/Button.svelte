<!-- src/lib/components/ui/Button.svelte -->

<script lang="ts">
  export let variant: 'primary' | 'secondary' | 'ghost' | 'danger' | 'outline' = 'secondary';
  export let size:    'sm' | 'md' | 'lg' = 'md';
  export let disabled  = false;
  export let loading   = false;
  export let fullWidth = false;
  export let type: 'button' | 'submit' | 'reset' = 'button';
</script>

<button
  {type}
  class="btn btn--{variant} btn--{size}"
  class:btn--full={fullWidth}
  class:btn--loading={loading}
  disabled={disabled || loading}
  on:click
  {...$$restProps}
>
  {#if loading}
    <span class="btn__spinner" aria-hidden="true"></span>
  {/if}
  <span class="btn__content" class:btn__content--hidden={loading}>
    <slot />
  </span>
</button>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .btn {
    @include m.flex-center;
    gap: v.$space-2;
    border-radius: v.$radius-md;
    font-family: v.$font-body;
    font-weight: v.$font-weight-semibold;
    white-space: nowrap;
    cursor: pointer;
    transition:
      background v.$transition-fast,
      color v.$transition-fast,
      border-color v.$transition-fast,
      box-shadow v.$transition-fast,
      opacity v.$transition-fast;
    position: relative;
    overflow: hidden;
    @include m.focus-ring;

    &:disabled {
      opacity: 0.45;
      cursor: not-allowed;
      pointer-events: none;
    }

    &--full { width: 100%; }

    // Sizes
    &--sm {
      height: 30px;
      padding: 0 v.$space-3;
      font-size: v.$font-size-sm;
    }
    &--md {
      height: 36px;
      padding: 0 v.$space-4;
      font-size: v.$font-size-base;
    }
    &--lg {
      height: 44px;
      padding: 0 v.$space-6;
      font-size: v.$font-size-md;
    }

    // Variants
    &--primary {
      background: var(--accent);
      color: var(--accent-fg);
      border: 1px solid transparent;
      &:hover:not(:disabled) {
        background: var(--accent-hover);
        box-shadow: 0 0 0 3px var(--accent-muted);
      }
    }

    &--secondary {
      background: var(--surface-2);
      color: var(--text-primary);
      border: 1px solid var(--border);
      &:hover:not(:disabled) {
        background: var(--surface-3);
        border-color: var(--border-hover);
      }
    }

    &--outline {
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--accent);
      &:hover:not(:disabled) {
        background: var(--accent-muted);
      }
    }

    &--ghost {
      background: transparent;
      color: var(--text-secondary);
      border: 1px solid transparent;
      &:hover:not(:disabled) {
        background: var(--nav-hover-bg);
        color: var(--text-primary);
      }
    }

    &--danger {
      background: rgba(239,68,68,0.12);
      color: var(--status-error);
      border: 1px solid rgba(239,68,68,0.2);
      &:hover:not(:disabled) {
        background: rgba(239,68,68,0.2);
        border-color: var(--status-error);
      }
    }

    // Spinner
    &__spinner {
      width: 14px;
      height: 14px;
      border: 2px solid currentColor;
      border-top-color: transparent;
      border-radius: 50%;
      animation: spin 0.7s linear infinite;
      flex-shrink: 0;
      position: absolute;
    }

    &__content {
      @include m.flex-center;
      gap: v.$space-2;
      &--hidden { visibility: hidden; }
    }

    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  }
</style>
