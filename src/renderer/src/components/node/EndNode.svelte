<script lang="ts">
  import Cross from "../icons/Cross.svelte";
  import { Handle, Position, useSvelteFlow, type NodeProps } from "@xyflow/svelte";

  const ICON_SIZE = 24;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  const { deleteElements } = useSvelteFlow();

  let { id }: NodeProps = $props();

  function deleteThisNode() {
    deleteElements({ nodes: [{ id }] });
  }
</script>

<Handle
  type="target"
  position={Position.Left}
  style={`${DEFAULT_HANDLE_STYLE}; background-color: var(--target-handle-color);`}
/>
<div {id} class="container">
  <p>End</p>
  <button onclick={deleteThisNode}>
    <Cross width={ICON_SIZE} height={ICON_SIZE} />
  </button>
</div>

<style>
  .container {
    align-items: center;
    display: flex;
    padding-inline: 1.2rem;
    gap: 1rem;
  }

  button {
    display: flex;
    align-items: center;
  }
</style>
