<script lang="ts">
  import AddDialogue from "../../../shared/components/icons/AddDialogue.svelte";

  import { dialogues } from "../../stores/dialogueStore.svelte";

  const ICON_SIZE = 24;

  function handleAddDialogue(): void {
    dialogues.add();
    dialogues.selectDialogue(dialogues.data.length - 1);
    dialogues.editing = true;
  }
</script>

<div class="controller">
  <button onclick={handleAddDialogue}>
    <AddDialogue width={ICON_SIZE} height={ICON_SIZE} color={"#322E2C"} />
  </button>
</div>

<style>
  button {
    padding: 0.3rem;
    border-radius: 0.3rem;
    transition: background-color ease-in 100ms;
  }

  button:hover {
    background-color: var(--secondary-color);
  }

  .controller {
    display: flex;
    justify-content: center;
    margin-bottom: 1rem;
  }
</style>
