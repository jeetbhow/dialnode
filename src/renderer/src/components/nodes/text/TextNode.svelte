<script lang="ts">
  import { useSvelteFlow, Handle, Position, type NodeProps } from "@xyflow/svelte";

  import type { DbEntityKind, TextNodeType } from "../../../../../shared/types";
  import { requestModal } from "../../../stores/dbModal.svelte";
  import TextNodeSettingsMenu from "./TextNodeSettingsMenu.svelte";
  import Cross from "../../icons/Cross.svelte";
  import Plus from "../../icons/Plus.svelte";
  import Minus from "../../icons/Minus.svelte";

  const ICON_SIZE = 18;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";
  const HANDLE_ID = {
    leftTarget: "left-target",
    topTarget: "top-target",
    rightSource: "right-source",
    bottomSource: "bottom-source"
  };

  let { id, data }: NodeProps<TextNodeType> = $props();
  let isSpeakerFieldEnabled: boolean = $state(false);
  let isPortraitFieldEnabled: boolean = $state(false);
  let currActiveHandleId: string = $state();

  const { updateNodeData, deleteElements } = useSvelteFlow();

  async function requestData(type: DbEntityKind): Promise<void> {
    const response = await requestModal(id, type);
    if (response === null || response.request.nodeId !== id) return;

    switch (type) {
      case "portrait":
        updateNodeData(id, { portrait: response.value });
        break;
      case "speaker":
        updateNodeData(id, { speaker: response.value });
        break;
    }
  }

  function removeData(type: DbEntityKind): void {
    switch (type) {
      case "portrait":
        updateNodeData(id, { portrait: null });
        break;
      case "speaker":
        updateNodeData(id, { speaker: null });
        break;
    }
  }

  function handleTextAreaChange(event: Event): void {
    const textarea = event.target as HTMLTextAreaElement;
    if (textarea) {
      /**
       * Setting the height to "auto" allows the textarea to shrink
       * to its "natural" height, so that the client height is equal to the content height.
       */
      textarea.style.height = "auto";
      textarea.style.height = `${textarea.scrollHeight}px`;
    }
    data.text = textarea.value;
  }
</script>

<Handle
  id={HANDLE_ID.leftTarget}
  type="target"
  position={Position.Left}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--target-handle-color);"
/>
<Handle
  id={HANDLE_ID.topTarget}
  type="target"
  position={Position.Top}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--target-handle-color)"
/>
{#if !currActiveHandleId || currActiveHandleId == HANDLE_ID.rightSource}
  <Handle
    id={HANDLE_ID.rightSource}
    type="source"
    position={Position.Right}
    style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
    onconnect={() => (currActiveHandleId = HANDLE_ID.rightSource)}
    ondisconnect={() => (currActiveHandleId = null)}
  />
{/if}
{#if !currActiveHandleId || currActiveHandleId == HANDLE_ID.bottomSource}
  <Handle
    id={HANDLE_ID.bottomSource}
    type="source"
    position={Position.Bottom}
    style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
    onconnect={() => (currActiveHandleId = HANDLE_ID.bottomSource)}
    ondisconnect={() => (currActiveHandleId = null)}
  />
{/if}
<div {id} class="container">
  <div class="header">
    <TextNodeSettingsMenu
      bind:isSpeakerEnabled={isSpeakerFieldEnabled}
      bind:isPortraitEnabled={isPortraitFieldEnabled}
    />
    <button class="cross" onclick={() => deleteElements({ nodes: [{ id }] })}>
      <Cross />
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
            <button class="field-btn" onclick={() => removeData("portrait")}>
              <Minus width={ICON_SIZE} height={ICON_SIZE} />
            </button>
          </div>
        {:else}
          <p>Portrait:</p>
          <button class="field-btn" onclick={() => requestData("portrait")}>
            <Plus width={ICON_SIZE} height={ICON_SIZE} />
          </button>
        {/if}
      </div>
    {/if}
    {#if isSpeakerFieldEnabled}
      <div class="field">
        {#if data.speaker}
          <div class="speaker-view">
            <p>{data.speaker.name}</p>
            <button class="field-btn" onclick={() => removeData("speaker")}>
              <Minus width={ICON_SIZE} height={ICON_SIZE} />
            </button>
          </div>
        {:else}
          <p>Speaker:</p>
          <button class="field-btn" onclick={() => requestData("speaker")}>
            <Plus width={ICON_SIZE} height={ICON_SIZE} />
          </button>
        {/if}
      </div>
    {/if}
    <div>
      <textarea
        id="text"
        class="nodrag"
        placeholder="Enter the dialogue text."
        oninput={handleTextAreaChange}
        value={data.text}
      ></textarea>
    </div>
  </form>
</div>

<style>
  p {
    margin: 0;
  }

  textarea {
    width: 100%;
    box-sizing: border-box;
    resize: none;
    overflow: hidden;
    padding: 0.5rem;
    border-radius: 0.2rem;
    border: 1px solid var(--textfield-border-color);
    background-color: var(--textfield-bg-color);
    font-family: var(--textfield-font-family);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
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

  .field-btn {
    display: flex;
    align-items: center;
    padding: 0.2rem;
    border-radius: 100%;
  }

  .field-btn:hover {
    background-color: var(--primary-color);
  }

  .portrait-overlay {
    position: relative;
    margin: auto;
  }

  .portrait-overlay img {
    display: block;
  }

  .portrait-overlay button {
    position: absolute;
    top: 0.2rem;
    right: 0.2rem;
    padding: 0.2rem;
    border-radius: 100%;
    background-color: var(--transparent-color);
    left: auto;
    z-index: 1;
    opacity: 0;
    transition: opacity 200ms ease-in-out;
  }

  .portrait-overlay:hover button {
    opacity: 1;
  }

  .speaker-view {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
  }
</style>
