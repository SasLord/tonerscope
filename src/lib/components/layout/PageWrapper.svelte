<!-- src/lib/components/layout/PageWrapper.svelte -->

<script lang="ts">
  export let padded = true;
  export let scrollable = true;
  export let maxWidth = false;
</script>

<main
  class="page"
  class:page--padded={padded}
  class:page--scrollable={scrollable}
  class:page--max={maxWidth}
>
  <slot />
</main>

<style lang="scss">
  @use '$lib/styles/variables' as v;
  @use '$lib/styles/mixins' as m;

  .page {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;

    &--padded { padding: v.$space-6; }

    &--scrollable {
      overflow-y: auto;
      @include m.custom-scrollbar;
    }

    &--max > :global(*) {
      max-width: v.$content-max-width;
      width: 100%;
      margin-inline: auto;
    }
  }
</style>
