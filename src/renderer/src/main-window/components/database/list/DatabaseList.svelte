<script lang="ts">
  import DatabaseListButton from "./DatabaseListButton.svelte";
  import type { DbEntity } from "../../../../../../shared/types";

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
