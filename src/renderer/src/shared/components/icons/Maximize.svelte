<script lang="ts">
  type Props = {
    width: number;
    height: number;
    color?: string;
  };

  const { width, height, color }: Props = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" {width} {height} viewBox="0 0 16 16"
  ><path fill={color ?? "currentColor"} d="M3 3v10h10V3zm9 9H4V4h8z" /></svg
>
