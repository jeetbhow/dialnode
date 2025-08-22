<script lang="ts">
  import type { Button } from "../../../../../shared/types";

  import MainButton from "./MainButton.svelte";

  export let flexDirection: "row" | "column";
  export let buttons: Button[];
</script>

<div style="flex-direction: {flexDirection}">
  {#each buttons as { text, onClick } (text)}
    <MainButton {text} {onClick} />
  {/each}
</div>

<style>
  div {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
