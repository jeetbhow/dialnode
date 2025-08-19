<script lang="ts">
  type Props = {
    width?: number;
    height?: number;
    rotated: boolean;
  };
  let { width, height, rotated }: Props = $props();
</script>

<svg class:rotated xmlns="http://www.w3.org/2000/svg" {width} {height} viewBox="0 0 24 24"
  ><path fill="currentColor" d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6l-6 6z" /></svg
>

<style>
  svg {
    transition: transform 500ms ease;
  }

  .rotated {
    transform: rotate(180deg);
  }
</style>
