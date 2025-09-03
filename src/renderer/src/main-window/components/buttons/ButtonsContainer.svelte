<script lang="ts">
  import type { Button } from "../../utils/buttons";

  import MainButton from "./MainButton.svelte";

  export let flexDirection: "row" | "column";
  export let buttons: Button[];
</script>

<div style="flex-direction: {flexDirection}">
  {#each buttons as { text, onClick, nodeType } (text)}
    <MainButton {text} {onClick} {nodeType} />
  {/each}
</div>

<style>
  div {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
