<!-- src/lib/components/ui/Tooltip.svelte -->

<script lang="ts">
  export let text     = '';
  export let position: 'top' | 'bottom' | 'left' | 'right' = 'top';
  export let delay    = 300;

  let visible = false;
  let timer: ReturnType<typeof setTimeout>;

  function show() { timer = setTimeout(() => visible = true, delay); }
  function hide() { clearTimeout(timer); visible = false; }
</script>

<span
  class="tooltip-host"
  on:mouseenter={show}
  on:mouseleave={hide}
  on:focusin={show}
  on:focusout={hide}
  role="none"
>
  <slot />
  {#if visible && text}
    <span class="tooltip tooltip--{position}" role="tooltip">{text}</span>
  {/if}
</span>

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .tooltip-host {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .tooltip {
    position: absolute;
    z-index: v.$z-dropdown;
    background: var(--surface-3);
    color: var(--text-primary);
    font-size: v.$font-size-xs;
    font-family: v.$font-mono;
    padding: 4px 8px;
    border-radius: v.$radius-md;
    border: 1px solid var(--border);
    box-shadow: v.$shadow-md;
    white-space: nowrap;
    pointer-events: none;
    animation: fadeIn 150ms ease;

    &--top    { bottom: calc(100% + 6px); left: 50%; transform: translateX(-50%); }
    &--bottom { top: calc(100% + 6px);    left: 50%; transform: translateX(-50%); }
    &--left   { right: calc(100% + 6px);  top: 50%;  transform: translateY(-50%); }
    &--right  { left: calc(100% + 6px);   top: 50%;  transform: translateY(-50%); }

    @keyframes fadeIn {
      from { opacity: 0; transform: translateX(-50%) translateY(3px); }
      to   { opacity: 1; transform: translateX(-50%) translateY(0); }
    }
  }
</style>
