<!-- src/lib/components/ui/Toast.svelte -->

<script lang="ts">
  import { notifications } from '$lib/stores/notifications';
  import { fly, fade } from 'svelte/transition';
  import { flip } from 'svelte/animate';

  const icons: Record<string, string> = {
    success: '✓',
    error:   '✕',
    warning: '⚠',
    info:    'ℹ',
  };
</script>

<div class="toast-container" aria-live="polite" aria-atomic="false">
  {#each $notifications as toast (toast.id)}
    <div
      class="toast toast--{toast.type}"
      in:fly={{ x: 20, duration: 250 }}
      out:fade={{ duration: 180 }}
      animate:flip={{ duration: 200 }}
      role="alert"
    >
      <span class="toast__icon">{icons[toast.type]}</span>
      <div class="toast__body">
        <p class="toast__title">{toast.title}</p>
        {#if toast.message}
          <p class="toast__message">{toast.message}</p>
        {/if}
      </div>
      <button class="toast__close" on:click={() => notifications.remove(toast.id)} aria-label="Закрыть">✕</button>
    </div>
  {/each}
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .toast-container {
    position: fixed;
    bottom: v.$space-6;
    right: v.$space-6;
    z-index: v.$z-toast;
    display: flex;
    flex-direction: column;
    gap: v.$space-2;
    pointer-events: none;
    max-width: 360px;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    gap: v.$space-3;
    padding: v.$space-3 v.$space-4;
    border-radius: v.$radius-lg;
    border: 1px solid;
    backdrop-filter: blur(12px);
    pointer-events: all;
    box-shadow: v.$shadow-lg;

    &--success {
      background: rgba(22,163,74,0.12);
      border-color: rgba(34,197,94,0.3);
      .toast__icon { color: var(--status-online); }
    }
    &--error {
      background: rgba(220,38,38,0.12);
      border-color: rgba(239,68,68,0.3);
      .toast__icon { color: var(--status-error); }
    }
    &--warning {
      background: rgba(217,119,6,0.12);
      border-color: rgba(245,158,11,0.3);
      .toast__icon { color: var(--status-warning); }
    }
    &--info {
      background: rgba(37,99,235,0.12);
      border-color: rgba(59,130,246,0.3);
      .toast__icon { color: var(--status-printing); }
    }

    &__icon {
      font-size: v.$font-size-md;
      font-weight: v.$font-weight-bold;
      flex-shrink: 0;
      margin-top: 1px;
    }

    &__body { flex: 1; min-width: 0; }

    &__title {
      font-size: v.$font-size-sm;
      font-weight: v.$font-weight-semibold;
      color: var(--text-primary);
      line-height: 1.3;
    }

    &__message {
      font-size: v.$font-size-xs;
      color: var(--text-secondary);
      margin-top: 2px;
      line-height: 1.4;
    }

    &__close {
      flex-shrink: 0;
      color: var(--text-tertiary);
      font-size: v.$font-size-xs;
      padding: 2px 4px;
      border-radius: v.$radius-sm;
      transition: color v.$transition-fast;
      &:hover { color: var(--text-primary); }
    }
  }
</style>
