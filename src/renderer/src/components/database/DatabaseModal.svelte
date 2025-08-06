<script lang="ts">
  import { modal, cancelModal, fufillModal } from "../../stores/dbModal.svelte";
  import type { DbRequestType, Portrait, Speaker } from "../../utils/types";

  import DatabaseListButton from "./DatabaseListButton.svelte";
  import Cross from "../icons/Cross.svelte";

  type Props = {
    portraits: Portrait[];
    speakers: Speaker[];
    projectDir: string;
  };

  let {
    portraits = $bindable(),
    speakers = $bindable(),
    projectDir = $bindable()
  }: Props = $props();

  const initialSelectedTab: DbRequestType = modal.requestType ?? "Portrait";

  let selectedId: string = $state();
  let newDbEntry: string = $state();
  let currSelectedTab: string = $state(initialSelectedTab);
  let selectedPortrait: Portrait = $derived(portraits.find((p) => p.id === selectedId) ?? null);
  let selectedSpeaker: Speaker = $derived(speakers.find((s) => s.id === selectedId) ?? null);

  function selectDbElem(id: string): void {
    selectedId = id;
  }

  function deleteDbElem(id: string): void {
    if (currSelectedTab === "Portrait") {
      portraits = portraits.filter((portrait) => portrait.id !== id);
    } else {
      speakers = speakers.filter((speaker) => speaker.id !== id);
    }
  }

  async function addPortrait() {
    if (!newDbEntry || newDbEntry === "") {
      return;
    }

    if (!projectDir || projectDir === "") {
      alert("A project directory must be set before you can add portraits.");
      return;
    }

    const id = crypto.randomUUID();
    const metadata = await window.api.selectImage(projectDir);

    if (!metadata.path.startsWith(projectDir)) {
      alert("The selected portrait is not contained in the project.");
      return;
    }

    const virtualPath = "res://" + metadata.relPath.replaceAll("\\", "/");

    const newPortrait = {
      id,
      name: newDbEntry,
      virtualPath,
      ...metadata
    };

    portraits.push(newPortrait);
    newDbEntry = null;
  }

  async function addSpeaker() {
    const id = crypto.randomUUID();

    const newSpeaker = {
      id,
      name: newDbEntry
    };

    speakers.push(newSpeaker);
  }
</script>

<div class="container modal">
  <div class="content">
    <header>
      <div class="tabs">
        <button
          disabled={modal.requestType === "Speaker"}
          class:selected={currSelectedTab === "Portrait"}
          onclick={() => (currSelectedTab = "Portrait")}>Portrait</button
        >
        <button
          disabled={modal.requestType === "Portrait"}
          class:selected={currSelectedTab === "Speaker"}
          onclick={() => (currSelectedTab = "Speaker")}>Speaker</button
        >
      </div>
      <button onclick={cancelModal}>
        <Cross />
      </button>
    </header>
    <p class="project-dir">Project: {projectDir}</p>
    <div class="view">
      <ul class="list">
        {#if currSelectedTab === "Portrait"}
          {#each portraits as portrait}
            <DatabaseListButton
              id={portrait.id}
              text={portrait.name}
              isSelected={portrait.id == selectedId}
              onSelect={selectDbElem}
              onDelete={deleteDbElem}
            />
          {/each}
        {:else}
          {#each speakers as speaker}
            <DatabaseListButton
              id={speaker.id}
              text={speaker.name}
              isSelected={speaker.id == selectedId}
              onSelect={selectDbElem}
              onDelete={deleteDbElem}
            />
          {/each}
        {/if}
      </ul>
      <div class="info">
        {#if selectedPortrait}
          <img
            src={selectedPortrait.dataURL}
            alt={selectedPortrait.name}
            width="100"
            height="100"
          />
          <p>{selectedPortrait.filename}</p>
          <p>{selectedPortrait.width} x {selectedPortrait.height}</p>
          <p>{selectedPortrait.virtualPath}</p>
        {/if}
      </div>
    </div>
    <div class="controls">
      <input type="text" bind:value={newDbEntry} />
      <button onclick={currSelectedTab === "Portrait" ? addPortrait : addSpeaker}>Add</button>
      {#if modal.nodeId}
        <button
          onclick={() =>
            fufillModal(currSelectedTab === "Portrait" ? selectedPortrait : selectedSpeaker)}
          >Select</button
        >
      {/if}
    </div>
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    flex-flow: row;
    padding: 0.5rem 0;
  }

  .controls {
    display: flex;
    gap: 0.3rem;
  }

  .controls button {
    font-size: 0.9rem;
    color: var(--button-text-color);
    background-color: var(--primary-color);
    padding: 0.5rem 1.7rem;
    border-radius: 0.2rem;
  }

  .controls input {
    font-size: 0.9rem;
    padding: 0.5rem;
    border-radius: 0.2rem;
    border: 1px solid var(--textfield-border-color);
    background-color: var(--textfield-bg-color);
    color: var(--text-color);
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    width: 50%;
    height: 60%;
    transform: translate(-50%, -50%);
    z-index: 1000;
  }

  .tabs button {
    margin-right: 0.5rem;
    padding: 0.3rem;
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-color);
  }

  .tabs button.selected {
    color: var(--primary-color);
    border-bottom: 1px solid var(--tab-bottom-border-color);
  }

  .tabs button[disabled] {
    color: var(--tab-disabled-color);
    cursor: default;
  }

  .content {
    display: flex;
    flex-flow: column;
    justify-content: space-between;
    box-sizing: border-box;
    padding: 1rem;
    height: 100%;
  }

  .project-dir {
    margin-bottom: 1rem;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 0 0 40%;
    overflow-y: auto;
  }

  .info {
    display: flex;
    flex-flow: column;
    align-items: center;
    justify-content: center;
    gap: 0.1rem;
    flex: 2 2 auto;
  }

  p {
    margin: 0;
    font-size: 0.7rem;
    color: var(--small-text-color);
  }

  .view {
    display: flex;
    flex-flow: row;
    justify-content: space-between;
    gap: 1rem;
    flex: 1 1 auto;
  }
</style>
