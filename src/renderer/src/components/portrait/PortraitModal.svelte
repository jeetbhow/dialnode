<script lang="ts">
  import cross from '../../assets/cross.svg';
  import type { Portrait } from '../../types';

  import PortraitListElement from './PortraitListElement.svelte';

  type Props = {
    portraits: Portrait[];
    onClose: () => void;
  };

  let { portraits = $bindable(), onClose }: Props = $props();

  let selectedId: string = $state();
  let newPortraitName: string = $state();
  let projectDir: string = $state();

  let selectedPortrait: Portrait = $derived(portraits.find((p) => p.id === selectedId) ?? null);

  function selectPortrait(id: string) {
    selectedId = id;
  }

  function deletePortrait(id: string) {
    portraits = portraits.filter((portrait) => portrait.id != id);
  }

  async function addPortrait() {
    const metadata = await window.api.selectImage();
    const id = crypto.randomUUID();

    const newPortrait = {
      id,
      name: newPortraitName,
      ...metadata
    };

    portraits.push(newPortrait);
    newPortraitName = '';
  }
</script>

<div class="modal">
  <div class="content">
    <header>
      <h2>Portraits</h2>
      <button onclick={onClose}>
        <img src={cross} alt="Cross" width="24" height="24" />
      </button>
    </header>
    <div class="view">
      <ul class="list">
        {#each portraits as portrait}
          <PortraitListElement
            id={portrait.id}
            text={portrait.name}
            isSelected={portrait.id == selectedId}
            onSelect={selectPortrait}
            onDelete={deletePortrait}
          />
        {/each}
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
        {/if}
      </div>
    </div>
    <div class="controls">
      <input type="text" bind:value={newPortraitName} />
      <button onclick={addPortrait}>Add Portrait</button>
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

  header button {
    background: none;
    border: none;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    width: 50%;
    height: 70%;
    transform: translate(-50%, -50%);
    background-color: rgb(255, 255, 255);
    padding: 0 0.8rem;
    border-radius: 0.8rem;
    box-shadow: 0 2px 10px rgba(124, 119, 119, 0.1);
    z-index: 1000;
  }

  .content {
    display: flex;
    flex-flow: column;
    justify-content: space-between;
    box-sizing: border-box;
    padding: 0.8rem;
    height: 100%;
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

  .info p {
    margin: 0;
  }

  .view {
    display: flex;
    flex-flow: row;
    justify-content: space-between;
    gap: 1rem;
    flex: 1 1 auto;
  }
</style>
