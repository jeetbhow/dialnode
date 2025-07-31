<script lang="ts">
  import type { DialogueNodeType } from '../../types'
  import cross from '../../assets/cross.svg'

  import { useSvelteFlow, type NodeProps } from '@xyflow/svelte'

  import DialogueNodeDropDown from './DialogueNodeDropDown.svelte'

  let { id, data }: NodeProps<DialogueNodeType> = $props()
  let isSpeakerFieldEnabled: boolean = $state(false)
  let isPortraitFieldEnabled: boolean = $state(false)

  const { updateNodeData, deleteElements } = useSvelteFlow()

  const autoResize = (event: Event) => {
    const textarea = event.target as HTMLTextAreaElement
    if (textarea) {
      textarea.style.height = 'auto'
      textarea.style.height = `${textarea.scrollHeight}px`
    }
  }
</script>

<div {id} class="node-container">
  <div class="header">
    <DialogueNodeDropDown
      bind:isSpeakerEnabled={isSpeakerFieldEnabled}
      bind:isPortraitEnabled={isPortraitFieldEnabled}
    />
    <button class="cross" onclick={() => deleteElements({ nodes: [{ id }] })}>
      <img src={cross} alt="delete" width="24" height="24" />
    </button>
  </div>

  <form
    onsubmit={(e) => {
      e.preventDefault()
    }}
  >
    {#if isSpeakerFieldEnabled}
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
    {/if}
    {#if isPortraitFieldEnabled}
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
    {/if}
    <div>
      <label for="text">Text:</label>
      <textarea id="text" class="nodrag" placeholder="Enter the dialogue text." oninput={autoResize}
      ></textarea>
    </div>
  </form>
</div>

<style>
  :root {
    --node-cross-bg-color: #ef5454;
    --node-bg-color: #ffffff;
  }

  label {
    display: block;
  }

  textarea {
    resize: none;
    overflow: hidden;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .node-container {
    background: var(--node-bg-color);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    padding: 0.5rem;
    border-radius: 0.3rem;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .cross {
    border-radius: 0.2rem;
    background: none;
    border: none;
  }

  .cross:hover {
    background-color: var(--node-cross-bg-color);
  }
</style>
