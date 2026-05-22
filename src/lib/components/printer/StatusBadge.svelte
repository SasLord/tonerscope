<!-- src/lib/components/printer/StatusBadge.svelte -->

<script lang="ts">
  import type { PrinterStatus } from '$lib/types/printer';
  import { statusLabel } from '$lib/utils/formatters';

  export let status: PrinterStatus;
  export let showDot = true;
  export let size: 'sm' | 'md' = 'md';

  type BadgeVariant = 'success' | 'error' | 'warning' | 'info' | 'neutral' | 'default';

  const variantMap: Record<PrinterStatus, BadgeVariant> = {
    online:   'success',
    offline:  'neutral',
    printing: 'info',
    error:    'error',
    warning:  'warning',
    unknown:  'default',
  };

  $: variant = variantMap[status] ?? 'default';
  $: isActive = status === 'online' || status === 'printing';
</script>

<span class="status-badge status-badge--{variant} status-badge--{size}">
  {#if showDot}
    <span class="status-badge__dot" class:status-badge__dot--pulse={isActive}></span>
  {/if}
  {statusLabel(status)}
</span>

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .status-badge {
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
      .status-badge__dot { width: 5px; height: 5px; }
    }
    &--md {
      font-size: v.$font-size-xs;
      padding: 2px 9px;
      .status-badge__dot { width: 6px; height: 6px; }
    }

    &__dot {
      border-radius: 50%;
      flex-shrink: 0;
      background: currentColor;

      &--pulse {
        position: relative;
        &::before {
          content: '';
          position: absolute;
          inset: 0;
          border-radius: 50%;
          background: currentColor;
          animation: ping 1.8s ease-in-out infinite;
          opacity: 0.6;
        }
      }
    }

    &--success {
      background: rgba(34,197,94,0.10);
      color: var(--status-online);
      border-color: rgba(34,197,94,0.2);
    }
    &--info {
      background: rgba(59,130,246,0.10);
      color: var(--status-printing);
      border-color: rgba(59,130,246,0.2);
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
    &--neutral {
      background: rgba(113,113,122,0.10);
      color: var(--text-tertiary);
      border-color: rgba(113,113,122,0.2);
    }
    &--default {
      background: var(--surface-2);
      color: var(--text-tertiary);
      border-color: var(--border);
    }

    @keyframes ping {
      0%        { transform: scale(1); opacity: 0.6; }
      70%, 100% { transform: scale(2.2); opacity: 0; }
    }
  }
</style>
