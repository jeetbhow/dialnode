<script lang="ts">
  import AddDialogue from "../../../shared/components/icons/AddDialogue.svelte";
  import AddFolder from "../../../shared/components/icons/AddFolder.svelte";

  import { root, type Dialogue, DialogueFolder } from "../../stores/dialoguesStore.svelte";

  const ICON_SIZE = 24;

  function handleAddDialogue(): void {
    const newDialogue: Dialogue = {
      id: crypto.randomUUID(),
      parent: root,
      type: "dialogue",
      name: "Untitled",
      nodes: [],
      edges: []
    };

    root.add(newDialogue);
    root.select(newDialogue);
    root.editing = true;
  }

  function handleAddFolder(): void {
    const newFolder = new DialogueFolder(crypto.randomUUID(), "Untitled", null);
    root.add(newFolder);
    root.select(newFolder);
    root.editing = true;
  }
</script>

<div class="controller">
  <button onclick={handleAddDialogue}>
    <AddDialogue width={ICON_SIZE} height={ICON_SIZE} color="#322E2C" />
  </button>
  <button onclick={handleAddFolder}>
    <AddFolder width={ICON_SIZE} height={ICON_SIZE} color="#322E2C" />
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
