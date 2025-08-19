<script lang="ts">
import Cross from "../../../../shared/components/icons/Cross.svelte";

  import { useSvelteFlow, Handle, Position, type NodeProps } from "@xyflow/svelte";
  import type { BranchNodeType } from "../../../../../../shared/types";
  import { dialogues } from "../../../../stores/dialogueStore.svelte";

  const ICON_SIZE = 26;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  let { id, data }: NodeProps<BranchNodeType> = $props();
  const { deleteElements } = useSvelteFlow();

  function deleteBranch(): void {
    deleteElements({ nodes: [{ id }] });
  }

  function handleChange(e: Event): void {
    const target = e.target as HTMLInputElement;
    const newName = target.value;

    dialogues.nodes = dialogues.nodes.map((n) =>
      n.id === id ? { ...n, data: { ...n.data, name: newName } } : n
    );

    dialogues.saveSelectedDialogue();
  }
</script>

<Handle
  type="source"
  position={Position.Right}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
/>
<div {id} class="container">
  <div class="content">
    <header>
      <h3>Branch</h3>
      <button aria-label="Delete branch" onclick={deleteBranch}>
        <Cross width={ICON_SIZE} height={ICON_SIZE} />
      </button>
    </header>
    <div>
      <label for="text">Text:</label>
      <input id="text" value={data.name} placeholder="Branch name" onchange={handleChange} />
    </div>
  </div>
</div>

<style>
  .content {
    margin: 0.3rem;
  }

  label {
    display: block;
    font-size: 0.9rem;
    font-weight: 600;
  }

  header {
    display: flex;
    justify-content: space-between;
  }

  header h3 {
    margin: auto;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5rem;
    border-radius: 0.2rem;
    border: 1px solid #ccc;
    background-color: #f9f9f9;
    font-family: "Iosevka-Regular", monospace;
  }
</style>
