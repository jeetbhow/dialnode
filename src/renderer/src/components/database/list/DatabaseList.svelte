<script lang="ts">
  import { type DbEntity } from "../../../stores/dbStore.svelte";
  import DatabaseListButton from "./DatabaseListButton.svelte";

  type Props = {
    data: DbEntity[];
    kind: string;
  };

  let { data = $bindable(), kind }: Props = $props();
</script>

<div class={kind}>
  <ul>
    {#each data as entity (entity.id)}
      <DatabaseListButton {entity} />
    {/each}
  </ul>
</div>

<style>
  div {
    height: 100%;
    flex: 1 1 auto;
  }
</style>
