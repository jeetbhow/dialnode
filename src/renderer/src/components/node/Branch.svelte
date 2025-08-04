<script lang="ts">
  import { useSvelteFlow, Handle, Position, type NodeProps } from '@xyflow/svelte';
  import type { BranchType } from '../../types';

  const DEFAULT_HANDLE_STYLE = 'width: 0.5rem; height: 0.5rem';

  let { id }: NodeProps<BranchType> = $props();
  const { deleteElements } = useSvelteFlow();

  function deleteBranch() {
    deleteElements({ nodes: [{ id }] });
  }
</script>

<Handle
  type="source"
  position={Position.Right}
  style="{DEFAULT_HANDLE_STYLE}; background-color: blue;"
/>
<div {id} class="container">
  <label for="branch-name">Branch</label>
  <div class="view">
    <button aria-label="Delete branch" onclick={deleteBranch}>
      <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"
        ><path
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-width="1.5"
          d="m8.464 15.535l7.072-7.07m-7.072 0l7.072 7.07"
        /></svg
      ></button
    >
    <input name="branch-name" placeholder="Branch name" />
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
    font-family: 'Iosevka-Regular', monospace;
  }

  .container {
    background-color: rgb(255, 255, 255);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    border-radius: 0.5rem;
    padding: 0.3rem;
  }

  .view {
    display: flex;
  }

  button {
    all: unset;
  }
</style>
