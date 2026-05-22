<!-- src/lib/components/ui/Card.svelte -->

<script lang="ts">
  export let padding: 'none' | 'sm' | 'md' | 'lg' = 'md';
  export let hoverable = false;
  export let clickable  = false;
  export let selected   = false;
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="card card--pad-{padding}"
  class:card--hoverable={hoverable}
  class:card--clickable={clickable}
  class:card--selected={selected}
  on:click
  on:keydown
>
  <slot />
</div>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .card {
    @include m.card-base;
    box-shadow: var(--shadow-card);

    &--pad-none { padding: 0; }
    &--pad-sm   { padding: v.$space-4; }
    &--pad-md   { padding: v.$space-5; }
    &--pad-lg   { padding: v.$space-8; }

    &--hoverable {
      @include m.card-hover;
    }

    &--clickable {
      cursor: pointer;
      @include m.card-hover;
      @include m.focus-ring;
      &:active { transform: scale(0.995); }
    }

    &--selected {
      border-color: var(--accent) !important;
      box-shadow: 0 0 0 3px var(--accent-muted), var(--shadow-card);
    }
  }
</style>
