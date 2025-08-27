<script lang="ts">
  import Chevron from "../../../../shared/components/icons/Chevron.svelte";
  import DialogueSelectItem from "./DialogueSelectListItem.svelte";

  import { root, type DialogueSelectNode } from "../../../stores/dialoguesStore.svelte";

  const BASE_DIALOGUE_INDENTATION = 2.6;
  const BASE_FOLDER_INDENTATION = 1;
  const ICON_SIZE = 20;

  type Props = {
    node: DialogueSelectNode;
    recursionLevel: number;
  };

  let { node, recursionLevel }: Props = $props();
  let showDropDown = $state(false);

  const dialogueIndentation = BASE_DIALOGUE_INDENTATION + recursionLevel + "rem";
  const folderIndentation = BASE_FOLDER_INDENTATION + recursionLevel + "rem";

  function handleClickShow() {
    showDropDown = !showDropDown;
  }

  function handleDialogueClick() {
    if (node.type !== "dialogue") {
      return;
    }

    root.selectDialogue(node);
  }
</script>

{#if node.type === "folder"}
  <li class="folder">
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
  </li>
{/if}

{#if node.type === "dialogue"}
  <li class="dialogue {root.selectedDialogue?.id === node.id ? 'selected' : ''}">
    <button onclick={handleDialogueClick} style:padding-left={dialogueIndentation}>
      {node.name}
    </button>
  </li>
{/if}

<style>
  button {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.9rem;
    font-weight: 300;
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
</style>
