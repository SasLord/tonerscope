<!-- src/lib/components/ui/ProgressBar.svelte -->

<script lang="ts">
  export let value: number = 0;   // 0–100
  export let color = '';          // CSS color или пустая строка → авто
  export let size: 'xs' | 'sm' | 'md' = 'sm';
  export let animated = true;
  export let showValue = false;

  import { tonerColor } from '$lib/utils/colors';

  $: resolvedColor = color || tonerColor(value);
  $: clamped = Math.max(0, Math.min(100, value));
</script>

<div class="progress progress--{size}" role="progressbar" aria-valuenow={clamped} aria-valuemin={0} aria-valuemax={100}>
  <div
    class="progress__track"
  >
    <div
      class="progress__bar"
      class:progress__bar--animated={animated}
      style="width: {clamped}%; background: {resolvedColor};"
    ></div>
  </div>
  {#if showValue}
    <span class="progress__label">{clamped}%</span>
  {/if}
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;

  .progress {
    display: flex;
    align-items: center;
    gap: v.$space-2;
    width: 100%;

    &__track {
      flex: 1;
      background: var(--gauge-track);
      border-radius: v.$radius-full;
      overflow: hidden;
    }

    &__bar {
      border-radius: v.$radius-full;
      transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);

      &--animated {
        position: relative;
        &::after {
          content: '';
          position: absolute;
          inset: 0;
          background: linear-gradient(
            90deg,
            transparent 0%,
            rgba(255,255,255,0.18) 50%,
            transparent 100%
          );
          animation: shimmer 2s ease-in-out infinite;
        }
      }
    }

    &__label {
      font-family: v.$font-mono;
      font-size: v.$font-size-xs;
      font-weight: v.$font-weight-semibold;
      color: var(--text-secondary);
      min-width: 32px;
      text-align: right;
    }

    // Sizes
    &--xs .progress__track { height: 3px; }
    &--sm .progress__track { height: 5px; }
    &--md .progress__track { height: 8px; }

    @keyframes shimmer {
      0%   { transform: translateX(-100%); }
      100% { transform: translateX(200%); }
    }
  }
</style>
