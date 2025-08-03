<script lang="ts">
  import trash from '../../assets/trash.svg';

  export let id: string;
  export let text: string;
  export let isSelected: boolean;
  export let onSelect: (id: string) => void;
  export let onDelete: (id: string) => void;
</script>

<li class:selected={isSelected}>
  <button class="select" on:click={() => onSelect(id)}>{text}</button>
  <button class="delete" on:click={() => onDelete(id)}>
    <img src={trash} width="16" height="16" alt="Trash can icon." />
  </button>
</li>

<style>
  li {
    --background-color: white;
    --background-color-hover: #f5f5f5;
    --text-color: rgb(52, 52, 52);
    --delete-btn-bg: rgb(255, 255, 255);

    display: flex;
    justify-content: space-between;
    color: var(--text-color);
    background-color: var(--background-color);
    border-radius: 0.5rem;
    padding: 0.5rem;
  }

  li:hover {
    background-color: var(--background-color-hover);
  }

  button {
    all: unset;
    display: inline-block;
    cursor: pointer;
    border-radius: 0.5rem;
  }

  .selected {
    background-color: var(--background-color-hover);
  }

  .select {
    flex: 2 2 auto;
  }

  .delete {
    flex: 1 1 auto;
    text-align: center;
    background-color: var(--delete-btn-bg);
  }
</style>
