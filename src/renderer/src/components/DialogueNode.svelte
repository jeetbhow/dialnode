<script lang="ts">
  import { useSvelteFlow, type NodeProps } from '@xyflow/svelte'
  import type { DialogueNodeType } from '../types'

  import cross from '../assets/cross.svg'

  let { id, data }: NodeProps<DialogueNodeType> = $props()

  const { updateNodeData, deleteElements } = useSvelteFlow()
</script>

<div {id} class="node-container">
  <button class="cross" onclick={() => deleteElements({ nodes: [{ id }] })}>
    <img src={cross} alt="delete" width="24" height="24" />
  </button>

  <form
    onsubmit={(e) => {
      e.preventDefault()
    }}
  >
    <div>
      <label for="speaker">Speaker:</label>
      <input
        class="nodrag"
        id="speaker"
        type="text"
        value={data.speaker ?? ''}
        oninput={(e) => updateNodeData(id, { speaker: (e.target as HTMLInputElement).value })}
        placeholder="Enter speaker name."
      />
    </div>
    <div>
      <label for="portrait">Portrait:</label>
      <input
        class="nodrag"
        id="portrait"
        type="text"
        value={data.portrait ?? ''}
        oninput={(e) => updateNodeData(id, { portrait: (e.target as HTMLInputElement).value })}
        placeholder="Enter portrait directory."
      />
    </div>
    <div>
      <label for="text">Text:</label>
      <textarea id="text" class="nodrag" placeholder="Enter the dialogue text."></textarea>
    </div>
  </form>
</div>

<style>
  :root {
    --cross-bg-color: #d95d5e;
  }

  label {
    display: block;
  }

  textarea {
    resize: none;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .node-container {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    padding: 0.5rem;
    border-radius: 0.3rem;
  }

  .cross {
    display: flex;
    border-radius: 0.2rem;
    margin-left: auto;
    background: none;
    border: none;
    transition: background 0.2s ease-in-out;
  }

  .cross:hover {
    background-color: var(--cross-bg-color);
  }
</style>
