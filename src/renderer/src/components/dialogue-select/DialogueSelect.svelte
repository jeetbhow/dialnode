<script lang="ts">
  import { dialogues } from "../../stores/dialogueStore.svelte";

  type Props = {
    onSelect: (index: number) => void;
  };

  let { onSelect }: Props = $props();
</script>

<div class="sidebar">
  <ul class="project-dialogues">
    {#each dialogues.data as dialogue, i (i)}
      <li class:selected={dialogues.selected === i}>
        <button onclick={() => onSelect(i)}>{dialogue.name}</button>
      </li>
    {/each}
  </ul>
</div>

<style>
  .sidebar {
    background-color: #fafafa;
    width: 15%;
    min-width: 250px;
    height: 100%;
  }

  .project-dialogues {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .project-dialogues li {
    margin: 0.3rem;
    padding: 0.3rem;
  }

  .project-dialogues li:hover {
    background-color: rgb(223, 223, 223);
  }

  .project-dialogues button {
    width: 100%;
  }

  .selected {
    background-color: rgb(223, 223, 223);
  }
</style>
