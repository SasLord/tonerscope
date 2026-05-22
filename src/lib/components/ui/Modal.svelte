<!-- src/lib/components/ui/Modal.svelte -->

<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import Button from './Button.svelte';

  export let open  = false;
  export let title = '';
  export let size: 'sm' | 'md' | 'lg' = 'md';

  const dispatch = createEventDispatcher<{ close: void }>();

  function close() { dispatch('close'); }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="modal-backdrop"
    transition:fade={{ duration: 200 }}
    on:click={close}
    role="presentation"
  ></div>

  <!-- Dialog -->
  <div
    class="modal-wrap"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <div
      class="modal modal--{size}"
      transition:fly={{ y: 16, duration: 250 }}
    >
      <div class="modal__header">
        <h2 class="modal__title" id="modal-title">{title}</h2>
        <button class="modal__close" on:click={close} aria-label="Закрыть">✕</button>
      </div>

      <div class="modal__body">
        <slot />
      </div>

      {#if $$slots.footer}
        <div class="modal__footer">
          <slot name="footer" />
        </div>
      {/if}
    </div>
  </div>
{/if}

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.6);
    backdrop-filter: blur(4px);
    z-index: v.$z-overlay;
  }

  .modal-wrap {
    position: fixed;
    inset: 0;
    z-index: v.$z-modal;
    @include m.flex-center;
    padding: v.$space-4;
    pointer-events: none;
  }

  .modal {
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: v.$radius-xl;
    box-shadow: var(--shadow-modal);
    pointer-events: all;
    width: 100%;
    max-height: 90vh;
    display: flex;
    flex-direction: column;

    &--sm  { max-width: 400px; }
    &--md  { max-width: 560px; }
    &--lg  { max-width: 760px; }

    &__header {
      @include m.flex-between;
      padding: v.$space-5 v.$space-6;
      border-bottom: 1px solid var(--border);
      flex-shrink: 0;
    }

    &__title {
      font-size: v.$font-size-lg;
      font-weight: v.$font-weight-bold;
      color: var(--text-primary);
    }

    &__close {
      width: 28px;
      height: 28px;
      @include m.flex-center;
      border-radius: v.$radius-md;
      color: var(--text-tertiary);
      font-size: v.$font-size-sm;
      transition: color v.$transition-fast, background v.$transition-fast;
      &:hover {
        color: var(--text-primary);
        background: var(--nav-hover-bg);
      }
    }

    &__body {
      padding: v.$space-6;
      overflow-y: auto;
      flex: 1;
      @include m.custom-scrollbar;
    }

    &__footer {
      padding: v.$space-4 v.$space-6;
      border-top: 1px solid var(--border);
      display: flex;
      justify-content: flex-end;
      gap: v.$space-3;
      flex-shrink: 0;
    }
  }
</style>
