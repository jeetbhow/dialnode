<script lang="ts">
  type Props = {
    width?: number;
    height?: number;
    color?: string;
  };

  const DEFAULT_SIZE = 30;

  const { width, height, color }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  width={width ?? DEFAULT_SIZE}
  height={height ?? DEFAULT_SIZE}
  viewBox="0 0 24 24"
  ><path
    fill="none"
    stroke={color ?? "currentColor"}
    stroke-linecap="round"
    stroke-width="1.5"
    d="m8.464 15.535l7.072-7.07m-7.072 0l7.072 7.07"
  /></svg
>
