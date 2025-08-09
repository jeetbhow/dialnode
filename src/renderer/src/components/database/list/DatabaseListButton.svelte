<script lang="ts">
  import { deleteEntity, selectEntity, useDb, type DbEntity } from "../../../stores/dbStore.svelte";
  import "../../../styles.css";

  import Trash from "../../icons/Trash.svelte";

  const ICON_SIZE = 16;

  type Props = {
    entity: DbEntity;
  };

  const db = useDb();

  let { entity }: Props = $props();

  function handleSelect() {
    selectEntity(entity);
  }

  function handleDelete() {
    deleteEntity(entity);
  }
</script>

<li class:selected={db.selectedId === entity.id}>
  <button class="select" onclick={handleSelect}>{entity.name}</button>
  <button class="delete" onclick={handleDelete}>
    <Trash width={ICON_SIZE} height={ICON_SIZE} />
  </button>
</li>

<style>
  li {
    display: flex;
    justify-content: space-between;
    color: var(--text-color);
    background-color: var(--node-bg-color);
    border-radius: 0.5rem;
    padding: 0.5rem;
  }

  li:hover {
    background-color: var(--secondary-color);
  }

  button {
    display: inline-block;
    border-radius: 0.5rem;
  }

  .selected {
    background-color: var(--secondary-color);
  }

  .select {
    flex: 5 2 auto;
  }

  .delete {
    flex: 0 0 4rem;
    text-align: center;
    background-color: var(--node-bg-color);
  }
</style>
