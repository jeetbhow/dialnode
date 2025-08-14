<script lang="ts">
  import Cross from "../icons/Cross.svelte";

  import { useSvelteFlow, Handle, Position, type NodeProps } from "@xyflow/svelte";
  import type { BranchNodeType } from "../../utils/types";

  const ICON_SIZE = 18;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  let { id, data }: NodeProps<BranchNodeType> = $props();
  const { deleteElements } = useSvelteFlow();

  function deleteBranch() {
    deleteElements({ nodes: [{ id }] });
  }

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    data.name = target.value;
  }
</script>

<Handle
  type="source"
  position={Position.Right}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
/>
<div {id} class="container">
  <label for="branch-name">Branch</label>
  <div>
    <button aria-label="Delete branch" onclick={deleteBranch}>
      <Cross width={ICON_SIZE} height={ICON_SIZE} />
    </button>
    <input name="branch-name" placeholder="Branch name" onchange={handleChange} />
  </div>
</div>

<style>
  label {
    font-size: 0.6rem;
    display: block;
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

  .container > div {
    display: flex;
  }
</style>
