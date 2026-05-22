<!-- src/lib/components/printer/TonerGauge.svelte -->

<script lang="ts">
  import type { Supply } from '$lib/types/printer';
  import { tonerColor } from '$lib/utils/colors';
  import { supplyLabel } from '$lib/utils/formatters';
  import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

  export let supply: Supply;
  export let compact = false;

  $: color = tonerColor(supply.percent);
  $: label = supplyLabel(supply.type);
</script>

<div class="gauge" class:gauge--compact={compact}>
  <div class="gauge__header">
    <div class="gauge__name-wrap">
      <span
        class="gauge__dot"
        style="background: {color};"
      ></span>
      <span class="gauge__name">{label}</span>
      {#if supply.isCritical}
        <span class="gauge__alert">КРИТИЧНО</span>
      {:else if supply.isLow}
        <span class="gauge__alert gauge__alert--low">МАЛО</span>
      {/if}
    </div>
    <span class="gauge__pct" style="color: {color};">{supply.percent}%</span>
  </div>
  <ProgressBar value={supply.percent} {color} size={compact ? 'xs' : 'sm'} />
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .gauge {
    display: flex;
    flex-direction: column;
    gap: v.$space-2;

    &--compact { gap: v.$space-1; }

    &__header {
      @include m.flex-between;
      gap: v.$space-2;
    }

    &__name-wrap {
      @include m.flex-start;
      gap: v.$space-2;
      min-width: 0;
    }

    &__dot {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      flex-shrink: 0;
    }

    &__name {
      font-size: v.$font-size-xs;
      color: var(--text-secondary);
      @include m.truncate;
    }

    &__pct {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-bold;
      flex-shrink: 0;
    }

    &__alert {
      font-family: v.$font-mono;
      font-size: 9px;
      font-weight: v.$font-weight-black;
      letter-spacing: 0.1em;
      padding: 1px 5px;
      border-radius: v.$radius-full;
      background: rgba(239,68,68,0.15);
      color: var(--status-error);
      border: 1px solid rgba(239,68,68,0.25);
      flex-shrink: 0;

      &--low {
        background: rgba(245,158,11,0.12);
        color: var(--status-warning);
        border-color: rgba(245,158,11,0.25);
      }
    }
  }
</style>
