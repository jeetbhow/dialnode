<script lang="ts">
  import DatabaseList from "../list/DatabaseList.svelte";

  import "./views.css";

  import { useDb, addEntity, type Speaker } from "../../../stores/dbStore.svelte";
  import { modal, fulfillModal } from "../../../stores/dbModal.svelte";

  const db = useDb();

  let newSpeakerName = $state("");
  const selectedSpeaker = $derived(db.speakers.find((p) => p.id === db.selectedId));

  function addSpeaker() {
    if (newSpeakerName === "") {
      return;
    }

    const newSpeaker: Speaker = {
      id: crypto.randomUUID(),
      kind: "speaker",
      name: newSpeakerName
    };

    addEntity(newSpeaker);
    newSpeakerName = "";
  }

  function handleSelect() {
    fulfillModal(selectedSpeaker);
  }
</script>

<div class="parent">
  <div class="view">
    <DatabaseList data={db.speakers} kind="speaker" />
  </div>

  {#if modal.requestType === "speaker"}
    <button onclick={handleSelect} class="select-btn secondary-btn">Select Speaker</button>
  {/if}

  <div class="controller">
    <div class="name">
      <input name="name" type="text" placeholder="Speaker Name" bind:value={newSpeakerName} />
      <button class="primary-btn" onclick={addSpeaker}>Add Speaker</button>
    </div>
  </div>
</div>

<style>
  .name {
    display: flex;
    text-align: center;
    gap: 0.3rem;
  }
</style>
