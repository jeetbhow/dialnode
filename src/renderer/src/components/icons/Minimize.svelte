<script lang="ts">
  type Props = {
    width: number;
    height: number;
    color?: string;
  };

  const { width, height, color }: Props = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" {width} {height} viewBox="0 0 16 16"
  ><path fill={color ?? "currentColor"} d="M14 8v1H3V8z" /></svg
>
