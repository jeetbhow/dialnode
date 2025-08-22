<script lang="ts">
  import { clickOutside } from "../../shared/attachments/attachments";
  import { dialogues } from "../../main-window/stores/dialogueStore.svelte";
  import { filterNodeProps } from "../../main-window/utils/utils";

  let open = $state(false);

  function handleClickSave() {
    dialogues.saveToDb();
  }

  function handleClickManageRepositories() {
    window.api.createRepositoryWindow();
  }

  async function handleClickExport() {
    const filteredNodes = filterNodeProps(dialogues.selectedNodes);
    await window.api.exportJson(filteredNodes);
  }
</script>

<div class="dropdown">
  <button onclick={() => (open = !open)} {@attach clickOutside(() => (open = false))}>File</button>
  {#if open}
    <ul class="dropdown-menu">
      <li>
        <button onclick={handleClickManageRepositories}>Manage Repositories</button>
      </li>
      <li>
        <button onclick={handleClickSave}>Save</button>
      </li>
      <li>
        <button onclick={handleClickExport}>Export</button>
      </li>
      <li>
        <button>Exit</button>
      </li>
    </ul>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .dropdown > button {
    padding: 0.4rem;
    margin: 0.3rem 0;
    border-radius: 0.5rem;
    font-size: 0.8rem;
  }

  .dropdown > button:hover {
    background: #ccc9c9;
  }

  .dropdown-menu {
    display: flex;
    flex-direction: column;
    list-style: none;
    position: absolute;
    top: 100%;
    left: 0;
    margin: 0;
    padding: 0;
    background: rgb(239, 237, 237);
    border: 1px solid #ccc;
    border-radius: 0.3rem;
    min-width: 400px;
    z-index: 1000;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .dropdown-menu li:hover {
    background: #ccc9c9;
  }

  .dropdown-menu li > button {
    box-sizing: border-box;
    padding: 0.2rem 3rem;
    width: 100%;
    font-size: 0.8rem;
  }
</style>
