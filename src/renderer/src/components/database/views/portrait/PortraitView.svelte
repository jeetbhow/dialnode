<script lang="ts">
  import Folder from "../../../icons/Folder.svelte";
  import { addEntity, useDb, type Portrait } from "../../../../stores/dbStore.svelte";
  import { useProject } from "../../../../stores/projectStore.svelte";
  import DatabaseList from "../../list/DatabaseList.svelte";
  import { fufillModal, modal } from "../../../../stores/dbModal.svelte";

  type ImageMetaData = {
    dataURL: string;
    width: number;
    height: number;
    type?: string;
    path: string;
    relPath: string;
    filename: string;
  };

  const ICON_SIZE = 18;
  const PORTRAIT_SIZE = 100;

  let db = useDb();
  const project = useProject();

  let imageMetaData = $state<ImageMetaData>(null);
  let newPortraitName = $state("");
  const selectedPortrait = $derived(db.portraits.find((p) => p.id === db.selectedId));

  async function setImageMetaData() {
    imageMetaData = await window.api.selectImage(project.dir);
  }

  function addPortrait() {
    if (!imageMetaData || newPortraitName === "") {
      return;
    }

    if (!project.dir) {
      alert("You must set a project directory before you can add a portrait.");
      return;
    }

    if (!imageMetaData.path.startsWith(project.dir)) {
      alert("The image is not contained in the project directory.");
      return;
    }

    const newPortrait: Portrait = {
      id: crypto.randomUUID(),
      name: newPortraitName,
      kind: "portrait",
      ...imageMetaData,
      virtualPath: "res://" + imageMetaData.relPath.replaceAll("\\", "/")
    };

    addEntity(newPortrait);
    newPortraitName = "";
  }

  function handleSelect() {
    fufillModal(selectedPortrait);
  }
</script>

<div class="parent">
  <div class="view">
    <DatabaseList data={db.portraits} kind="portrait" />
    <div class="portrait-info">
      {#if selectedPortrait}
        <img
          src={selectedPortrait?.dataURL}
          alt="Character portrait"
          width={PORTRAIT_SIZE}
          height={PORTRAIT_SIZE}
        />
        <p>{selectedPortrait?.filename}</p>
        <p>{selectedPortrait?.width} x {selectedPortrait?.height}</p>
        <p>{selectedPortrait?.virtualPath}</p>
      {/if}
    </div>
  </div>

  {#if modal.requestType === "portrait"}
    <button onclick={handleSelect} class="select-portrait-btn secondary-btn">Select Portrait</button
    >
  {/if}

  <div class="controller">
    <div class="path">
      <p>{imageMetaData?.filename ?? "Upload:"}</p>
      <button class="secondary-btn" onclick={setImageMetaData}>
        <Folder width={ICON_SIZE} height={ICON_SIZE} />
      </button>
    </div>
    <div class="name">
      <input name="name" type="text" placeholder="Portrait Name" bind:value={newPortraitName} />
      <button class="primary-btn" onclick={addPortrait}>Add Portrait</button>
    </div>
  </div>
</div>

<style>
  p {
    margin: 0;
    font-size: 0.8rem;
  }

  .select-portrait-btn {
    width: 25%;
    margin: 3rem auto;
  }

  .parent {
    display: flex;
    flex-flow: column;
    justify-content: space-between;
    height: 100%;
  }

  .controller {
    display: flex;
    gap: 0.9rem;
  }

  .controller div {
    display: flex;
    align-items: center;
  }

  .view {
    display: flex;
    box-sizing: border-box;
    height: 100%;
    width: 100%;
    padding: 2rem;
    gap: 1rem;
  }

  .portrait-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    width: 50%;
  }

  .path {
    gap: 0.9rem;
  }

  .name {
    gap: 0.3rem;
  }
</style>
