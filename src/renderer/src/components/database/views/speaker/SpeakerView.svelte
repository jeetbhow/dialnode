<script lang="ts">
  import DatabaseList from "../../list/DatabaseList.svelte";
  import { useDb, addEntity, type Speaker } from "../../../../stores/dbStore.svelte";
  import { modal, fufillModal } from "../../../../stores/dbModal.svelte";

  const db = useDb();

  let newSpeakerName = $state("");
  const selectedSpeaker = $derived(db.speakers.find((p) => p.id === db.selectedId));

  function addSpeaker() {
    const newSpeaker: Speaker = {
      id: crypto.randomUUID(),
      kind: "speaker",
      name: newSpeakerName
    };

    addEntity(newSpeaker);
  }

  function handleSelect() {
    fufillModal(selectedSpeaker);
  }
</script>

<div class="parent">
  <div class="view">
    <DatabaseList data={db.speakers} kind="speaker" />
  </div>

  {#if modal.requestType === "speaker"}
    <button onclick={handleSelect} class="select-speaker-btn secondary-btn">Select Speaker</button>
  {/if}

  <div class="controller">
    <div class="name">
      <input name="name" type="text" placeholder="Speaker Name" bind:value={newSpeakerName} />
      <button class="primary-btn" onclick={addSpeaker}>Add Speaker</button>
    </div>
  </div>
</div>

<style>
  .parent {
    display: flex;
    flex-flow: column;
    justify-content: space-between;
    height: 100%;
  }

  .view {
    display: flex;
    box-sizing: border-box;
    height: 100%;
    width: 100%;
    padding: 2rem;
  }

  .name {
    display: flex;
    text-align: center;
    gap: 0.3rem;
  }

  .select-speaker-btn {
    width: 25%;
    margin: 3rem auto;
  }
</style>
