<script lang="ts">
  import { Handle, NodeResizer, Position, useSvelteFlow, type NodeProps } from '@xyflow/svelte';
  import type { BranchNodeType } from '../../types';

  const DEFAULT_HANDLE_STYLE = 'width: 0.5rem; height: 0.5rem';

  let { id, data, selected }: NodeProps<BranchNodeType> = $props();
  const { getNode, deleteElements } = useSvelteFlow();

  function addBranch() {
    const thisNode = getNode(id);
    data.addBranch('branch', thisNode.id);
  }

  function addSkillCheck() {
    // TODO  Create a skill check node type and fill in this function.
  }
</script>

<NodeResizer
  autoScale={true}
  isVisible={selected}
  minWidth={240}
  minHeight={240}
  style="margin: 0.5rem"
/>
<Handle
  type="target"
  position={Position.Left}
  style="{DEFAULT_HANDLE_STYLE}; background-color:red;"
/>
<div {id} class="container">
  <div class="control-btns">
    <button aria-label="Add branch." onclick={addBranch}
      ><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
        ><path
          fill="currentColor"
          d="M18 13h-5v5c0 .55-.45 1-1 1s-1-.45-1-1v-5H6c-.55 0-1-.45-1-1s.45-1 1-1h5V6c0-.55.45-1 1-1s1 .45 1 1v5h5c.55 0 1 .45 1 1s-.45 1-1 1"
        /></svg
      ></button
    >
    <button aria-label="Add skill check node">
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
        ><path
          fill="currentColor"
          d="M18.333 2c1.96 0 3.56 1.537 3.662 3.472l.005.195v12.666c0 1.96-1.537 3.56-3.472 3.662l-.195.005H5.667a3.667 3.667 0 0 1-3.662-3.472L2 18.333V5.667c0-1.96 1.537-3.56 3.472-3.662L5.667 2zM15.5 14a1.5 1.5 0 1 0 0 3a1.5 1.5 0 0 0 0-3m-7 0a1.5 1.5 0 1 0 0 3a1.5 1.5 0 0 0 0-3m0-7a1.5 1.5 0 1 0 0 3a1.5 1.5 0 0 0 0-3m7 0a1.5 1.5 0 1 0 0 3a1.5 1.5 0 0 0 0-3"
        /></svg
      >
    </button>
  </div>
  <button aria-label="Delete node" onclick={() => deleteElements({ nodes: [{ id }] })}>
    <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 24 24"
      ><path
        fill="none"
        stroke="currentColor"
        stroke-linecap="round"
        stroke-width="1.5"
        d="m8.464 15.535l7.072-7.07m-7.072 0l7.072 7.07"
      /></svg
    >
  </button>
</div>

<style>
  .container {
    background-color: rgb(255, 255, 255);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    border-radius: 0.5rem;
    width: 100%;
    height: 100%;
    padding: 0.5rem;
    display: flex;
    align-items: start;
    justify-content: space-between;
  }

  .control-btns {
    display: flex;
    gap: 0.3rem;
  }

  button {
    all: unset;
    cursor: pointer;
    transition:
      background 150ms ease-in-out,
      color 150ms ease-in-out;
  }
</style>
