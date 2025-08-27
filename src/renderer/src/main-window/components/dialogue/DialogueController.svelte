<script lang="ts">
  import AddDialogue from "../../../shared/components/icons/AddDialogue.svelte";

  import { root, type DialogueSelectNode } from "../../stores/dialoguesStore.svelte";

  const ICON_SIZE = 24;

  function handleAddDialogue(): void {
    const newDialogue: DialogueSelectNode = {
      id: crypto.randomUUID(),
      type: "dialogue",
      name: "Untitled",
      selected: false,
      nodes: [],
      edges: []
    };

    root.add(newDialogue);
  }
</script>

<div class="controller">
  <button onclick={handleAddDialogue}>
    <AddDialogue width={ICON_SIZE} height={ICON_SIZE} color="#322E2C" />
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
