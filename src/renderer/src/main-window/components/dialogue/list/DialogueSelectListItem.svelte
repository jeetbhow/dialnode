<script lang="ts">
  import { tick } from "svelte";

  import Chevron from "../../../../shared/components/icons/Chevron.svelte";
  import DialogueSelectItem from "./DialogueSelectListItem.svelte";

  import { rootState, type DialogueSelectNode } from "../../../stores/dialoguesStore.svelte";
  import { graph } from "../../../stores/graphStore.svelte";

  const BASE_DIALOGUE_INDENTATION = 2.6;
  const BASE_FOLDER_INDENTATION = 1;
  const ICON_SIZE = 20;

  type Props = {
    node: DialogueSelectNode;
    recursionLevel: number;
  };

  let { node = $bindable(), recursionLevel }: Props = $props();
  let showDropDown = $state(false);
  let renameInputValue = $state(node.name);
  let draggedOn = $state(false);

  function autofocus(el: HTMLInputElement) {
    tick().then(() => {
      el.focus();
      el.select();
    });
  }

  const dialogueIndentation = BASE_DIALOGUE_INDENTATION + recursionLevel + "rem";
  const folderIndentation = BASE_FOLDER_INDENTATION + recursionLevel + "rem";

  function handleClickShow(): void {
    showDropDown = !showDropDown;
  }

  function handleDialogueClick(): void {
    if (node.type !== "dialogue") {
      return;
    }

    graph.select(node);
  }

  function handleSubmitRename(event: SubmitEvent) {
    event.preventDefault();
    node.name = renameInputValue;
    rootState.editing = false;
  }

  function handleRenameChange() {
    rootState.editing = false;
  }

  function handleDragStart(): void {
    rootState.draggedNode = node;
  }

  function handleDragOver(event: DragEvent): void {
    event.preventDefault();
    draggedOn = true;
  }

  function handleDragLeave(): void {
    draggedOn = false;
  }

  function handleDrop(): void {
    if (node.type !== "folder") {
      return;
    }

    const draggedEl = rootState.draggedNode;
    draggedEl.move(node);

    rootState.draggedNode = null;
    draggedOn = false;
  }
</script>

{#if node.type === "folder"}
  <li
    class="folder"
    draggable="true"
    ondragstart={handleDragStart}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
    class:dragged={draggedOn}
  >
    {#if rootState.editing && graph.selected.id === node.id}
      <form onsubmit={handleSubmitRename}>
        <input use:autofocus bind:value={renameInputValue} onblur={handleRenameChange} />
      </form>
    {:else}
      <button onclick={handleClickShow} style:padding-left={folderIndentation}>
        <Chevron width={ICON_SIZE} height={ICON_SIZE} direction={showDropDown ? "down" : "right"} />
        {node.name}
      </button>
      {#if showDropDown}
        <ul>
          {#each node.children as child}
            <DialogueSelectItem node={child} recursionLevel={recursionLevel + 1} />
          {/each}
        </ul>
      {/if}
    {/if}
  </li>
{/if}

{#if node.type === "dialogue"}
  <li
    class="dialogue {graph.selected?.id === node.id ? 'selected' : ''}"
    draggable="true"
    ondragstart={handleDragStart}
  >
    {#if rootState.editing && graph.selected.id === node.id}
      <form onsubmit={handleSubmitRename}>
        <input use:autofocus bind:value={renameInputValue} onblur={handleRenameChange} />
      </form>
    {:else}
      <button onclick={handleDialogueClick} style:padding-left={dialogueIndentation}>
        {node.name}
      </button>
    {/if}
  </li>
{/if}

<style>
  button {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.9rem;
    font-weight: 300;
    border: solid 2px transparent;
  }

  button:hover {
    background-color: var(--secondary-color);
  }

  ul {
    list-style: none;
    padding: 0rem;
  }

  li button {
    display: flex;
    align-items: center;
    box-sizing: border-box;
    width: 100%;
    padding: 0.3rem 0;
    transition: background-color ease-in 100ms;
  }

  .dialogue.selected {
    background-color: var(--secondary-color);
  }

  li.dragged {
    border: solid 2px var(--primary-color);
  }

  form input {
    box-sizing: border-box;
    width: 100%;
  }

  form input:focus {
    outline-color: var(--primary-color);
  }
</style>
