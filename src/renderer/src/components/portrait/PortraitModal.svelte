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

  function selectPortrait(id: string) {
    selectedId = id;
  }

  function deletePortrait(id: string) {
    portraits = portraits.filter((portrait) => portrait.id != id);
  }

  async function addPortrait() {
    const dir = await window.api.selectImage();
    const id = crypto.randomUUID();

    const newPortrait = {
      id,
      name: newPortraitName,
      path: dir
    };
    portraits.push(newPortrait);
    newPortraitName = null;
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
        <p>This is where the portrait info goes.</p>
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
    flex-flow: row nowrap;
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
    height: 75%;
    transform: translate(-50%, -50%);
    background-color: rgb(255, 255, 255);
    padding: 0 0.8rem;
    border-radius: 0.8rem;
    box-shadow: 0 2px 10px rgba(124, 119, 119, 0.1);
    z-index: 1000;
  }

  .content {
    display: flex;
    flex-flow: column nowrap;
    justify-content: space-between;
    box-sizing: border-box;
    padding: 0.8rem;
    height: 100%;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    flex: 1 1 auto;
  }

  .info {
    flex: 2 2 auto;
  }

  .view {
    display: flex;
    flex-flow: row nowrap;
    justify-content: space-between;
    flex: 1 1 auto;
    gap: 1rem;
  }
</style>
