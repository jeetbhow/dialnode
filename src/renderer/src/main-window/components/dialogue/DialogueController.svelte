<script lang="ts">
  import AddDialogue from "../../../shared/components/icons/AddDialogue.svelte";
  import AddFolder from "../../../shared/components/icons/AddFolder.svelte";

  import { Dialogue, root, rootState, Folder } from "../../stores/dialoguesStore.svelte";
  import { graph } from "../../stores/graphStore.svelte";

  const ICON_SIZE = 24;

  function handleAddDialogue(): void {
    const newDialogue = new Dialogue();
    root.add(newDialogue);
    graph.select(newDialogue);
    rootState.editing = true;
  }

  function handleAddFolder(): void {
    const newFolder = new Folder();
    root.add(newFolder);
    graph.select(newFolder);
    rootState.editing = true;
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
