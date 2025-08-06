<script lang="ts">
  import "../../styles.css";

  import Trash from "../icons/Trash.svelte";

  const ICON_SIZE = 16;

  type Props = {
    id: string;
    text: string;
    isSelected: boolean;
    onSelect: (id: string) => void;
    onDelete: (id: string) => void;
  };

  let { id, text, isSelected, onSelect, onDelete }: Props = $props();
</script>

<li class:selected={isSelected}>
  <button class="select" onclick={() => onSelect(id)}>{text}</button>
  <button class="delete" onclick={() => onDelete(id)}>
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
    flex: 1 1 auto;
    text-align: center;
    background-color: var(--node-bg-color);
  }
</style>
