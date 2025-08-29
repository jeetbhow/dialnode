<script lang="ts">
  import { root } from "../../../stores/dialoguesStore.svelte";
  import DialogueSelectListItem from "./DialogueSelectListItem.svelte";
</script>

<div>
  <ul>
    {#each root.children as node}
      <li>
        <DialogueSelectListItem bind:node recursionLevel={0} />
      </li>
    {/each}
  </ul>
</div>

<style>
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
</style>
