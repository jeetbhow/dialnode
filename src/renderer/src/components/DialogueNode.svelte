<script lang="ts">
  import { useSvelteFlow, type NodeProps } from '@xyflow/svelte'
  import type { DialogueNodeType } from '../types'

  let { id, data }: NodeProps<DialogueNodeType> = $props()
  const { updateNodeData } = useSvelteFlow()
</script>

<div {id} class="node-container">
  <form
    onsubmit={(e) => {
      e.preventDefault()
    }}
  >
    <div>
      <label for="speaker">Speaker:</label>
      <input
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
        id="portrait"
        type="text"
        value={data.portrait ?? ''}
        oninput={(e) => updateNodeData(id, { portrait: (e.target as HTMLInputElement).value })}
        placeholder="Enter portrait directory."
      />
    </div>
    <div>
      <label for="text">Text:</label>
      <textarea id="text" placeholder="Enter the dialogue text."></textarea>
    </div>
  </form>
</div>

<style>
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
</style>
