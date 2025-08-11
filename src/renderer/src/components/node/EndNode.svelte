<script lang="ts">
  import { Handle, Position, type NodeProps } from "@xyflow/svelte";

  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  let { id }: NodeProps = $props();
</script>

<Handle
  type="target"
  position={Position.Left}
  style={`${DEFAULT_HANDLE_STYLE}; background-color: var(--target-handle-color);`}
/>
<div {id} class="container">End</div>

<style>
  div {
    font-size: 1.3rem;
    font-weight: 600;
    padding: 2rem 3rem;
  }
</style>
