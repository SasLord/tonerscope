<!-- src/lib/components/ui/Badge.svelte -->

<script lang="ts">
  export let variant: 'default' | 'success' | 'warning' | 'error' | 'info' | 'neutral' = 'default';
  export let size: 'sm' | 'md' = 'md';
  export let dot = false;
</script>

<span class="badge badge--{variant} badge--{size}" class:badge--dot={dot}>
  {#if dot}<span class="badge__dot" aria-hidden="true"></span>{/if}
  <slot />
</span>

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: v.$font-mono;
    font-weight: v.$font-weight-semibold;
    letter-spacing: 0.04em;
    border-radius: v.$radius-full;
    border: 1px solid;
    white-space: nowrap;

    &--sm {
      font-size: 10px;
      padding: 1px 7px;
    }
    &--md {
      font-size: v.$font-size-xs;
      padding: 2px 9px;
    }

    &__dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: currentColor;
      flex-shrink: 0;
    }

    // Variants
    &--default {
      background: var(--surface-2);
      color: var(--text-secondary);
      border-color: var(--border);
    }
    &--success {
      background: rgba(34,197,94,0.10);
      color: var(--status-online);
      border-color: rgba(34,197,94,0.2);
    }
    &--warning {
      background: rgba(245,158,11,0.10);
      color: var(--status-warning);
      border-color: rgba(245,158,11,0.2);
    }
    &--error {
      background: rgba(239,68,68,0.10);
      color: var(--status-error);
      border-color: rgba(239,68,68,0.2);
    }
    &--info {
      background: rgba(59,130,246,0.10);
      color: var(--status-printing);
      border-color: rgba(59,130,246,0.2);
    }
    &--neutral {
      background: rgba(113,113,122,0.10);
      color: var(--text-tertiary);
      border-color: rgba(113,113,122,0.2);
    }
  }
</style>
