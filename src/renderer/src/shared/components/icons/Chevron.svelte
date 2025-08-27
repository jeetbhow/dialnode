<script lang="ts">
  type Props = {
    width?: number;
    height?: number;
    direction: "left" | "right" | "up" | "down";
  };
  let { width, height, direction }: Props = $props();
</script>

<svg class={direction} xmlns="http://www.w3.org/2000/svg" {width} {height} viewBox="0 0 24 24">
  <path fill="currentColor" d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6l-6 6z" />
</svg>

<style>
  svg {
    transition: transform 100ms ease;
  }

  .up {
    transform: rotate(0deg);
  }

  .down {
    transform: rotate(180deg);
  }

  .left {
    transform: rotate(-90deg);
  }

  .right {
    transform: rotate(90deg);
  }
</style>
