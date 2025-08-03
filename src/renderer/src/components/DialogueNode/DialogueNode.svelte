<script lang="ts">
  import { useSvelteFlow, type NodeProps } from '@xyflow/svelte';

  import cross from '../../assets/cross.svg';
  import plus from '../../assets/plus.svg';
  import minus from '../../assets/minus.svg';
  import type { DialogueNodeType, DbRequestType } from '../../types';

  import { requestModal } from '../../stores/portraitModal.svelte';

  import DialogueNodeDropDown from './DialogueNodeDropDown.svelte';

  let { id, data }: NodeProps<DialogueNodeType> = $props();

  let isSpeakerFieldEnabled: boolean = $state(false);
  let isPortraitFieldEnabled: boolean = $state(false);

  const { updateNodeData, deleteElements } = useSvelteFlow();

  async function requestData(type: DbRequestType): Promise<void> {
    const response = await requestModal(id, type);
    if (response === null || response.nodeId !== id) return;

    if (type === 'Portrait') {
      updateNodeData(id, { portrait: response.value });
    } else {
      updateNodeData(id, { speaker: response.value });
    }
  }

  function removeData(type: DbRequestType): void {
    if (type === 'Portrait') {
      updateNodeData(id, { portrait: null });
    } else {
      updateNodeData(id, { speaker: null });
    }
  }

  function autoResize(event: Event): void {
    const textarea = event.target as HTMLTextAreaElement;
    if (textarea) {
      /**
       * Setting the height to "auto" allows the textarea to shrink
       * to its "natural" height, so that the client height is equal to the content height.
       */
      textarea.style.height = 'auto';
      textarea.style.height = `${textarea.scrollHeight}px`;
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
      e.preventDefault();
    }}
  >
    {#if isPortraitFieldEnabled}
      <div class="field">
        {#if data.portrait}
          <div class="portrait-overlay">
            <img
              src={data.portrait.dataURL}
              alt="A portrait of a character."
              width="50"
              height="50"
            />
            <button class="minus-btn" onclick={() => removeData('Portrait')}>
              <img src={minus} alt="Delete portrait." width="18" height="18" />
            </button>
          </div>
        {:else}
          <p>Portrait:</p>
          <button class="plus-btn" onclick={() => requestData('Portrait')}>
            <img src={plus} alt="Add portrait." width="16" height="16" />
          </button>
        {/if}
      </div>
    {/if}
    {#if isSpeakerFieldEnabled}
      <div class="field">
        {#if data.speaker}
          <div class="speaker-view">
            <p>{data.speaker.name}</p>
            <button class="minus-btn" onclick={() => removeData('Speaker')}>
              <img src={minus} alt="Delete portrait." width="16" height="16" />
            </button>
          </div>
        {:else}
          <p>Speaker:</p>
          <button class="plus-btn" onclick={() => requestData('Speaker')}>
            <img src={plus} alt="Add Speaker." width="16" height="16" />
          </button>
        {/if}
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
  p {
    margin: 0;
  }

  label {
    display: block;
  }

  textarea {
    width: 100%;
    box-sizing: border-box;
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

  button {
    all: unset;
    cursor: pointer;
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
  }

  .field {
    display: flex;
    gap: 0.5rem;
    margin: auto;
  }

  .plus-btn {
    display: flex;
    align-items: center;
    padding: 0.2rem;
    border-radius: 100%;
  }

  .plus-btn:hover {
    background-color: #8ec3f4;
  }

  .minus-btn {
    display: flex;
    align-items: center;
    padding: 0.2rem;
    border-radius: 100%;
  }

  .minus-btn:hover {
    background-color: #f46c6c;
  }

  .portrait-overlay {
    position: relative;
    margin: auto;
  }

  .portrait-overlay img {
    display: block;
  }

  .portrait-overlay button {
    all: unset;
    position: absolute;
    top: 0.2rem;
    right: 0.2rem;
    padding: 0.2rem;
    border-radius: 100%;
    background-color: rgba(219, 217, 217, 0.453);
    left: auto;
    z-index: 1;
    opacity: 0;
    transition: opacity 200ms ease-in-out;
  }

  .portrait-overlay button:hover {
    cursor: pointer;
  }

  .portrait-overlay:hover button {
    opacity: 1;
  }

  .speaker-view {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
  }

  .cross {
    border-radius: 0.2rem;
    background: none;
    border: none;
  }
</style>
