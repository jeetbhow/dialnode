<script lang="ts">
  import Cross from "../icons/Cross.svelte";
  import DatabaseTabs from "./tabs/DatabaseTabs.svelte";
  import PortraitView from "./views/portrait/PortraitView.svelte";

  import { modal, cancelModal } from "../../stores/dbModal.svelte";
  import { type DbEntityKind } from "../../stores/dbStore.svelte";
  import { useProject } from "../../stores/projectStore.svelte";
  import SpeakerView from "./views/speaker/SpeakerView.svelte";

  const project = useProject();
  const initialSelectedTab: DbEntityKind = modal.requestType ?? "portrait";

  let currSelectedTab: string = $state(initialSelectedTab);
</script>

<div class="container modal">
  <div class="content">
    <header>
      <DatabaseTabs bind:selected={currSelectedTab} />
      <button onclick={cancelModal}>
        <Cross />
      </button>
    </header>
    <p class="project-dir">Project: {project.dir}</p>
    {#if currSelectedTab === "portrait"}
      <PortraitView />
    {:else if currSelectedTab === "speaker"}
      <SpeakerView />
    {:else if currSelectedTab === "skill"}{/if}
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    flex-flow: row;
    padding: 0.5rem 0;
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

  .content {
    display: flex;
    flex-flow: column;
    box-sizing: border-box;
    padding: 1rem;
    height: 100%;
  }

  .project-dir {
    margin-bottom: 1rem;
  }

  p {
    margin: 0;
    font-size: 0.7rem;
    color: var(--small-text-color);
  }
</style>
