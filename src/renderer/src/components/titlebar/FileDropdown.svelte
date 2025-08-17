<script lang="ts">
  import { clickOutside } from "../../attachments/attachments";
  import { setProjectDirectory } from "../../stores/projectStore.svelte";

  let open = $state(false);

  async function handleClickNewProject(): Promise<void> {
    const projectDir = await window.api.selectDirectory();
    setProjectDirectory(projectDir);
  }
</script>

<div class="dropdown">
  <button onclick={() => (open = !open)} {@attach clickOutside(() => (open = false))}>File</button>
  {#if open}
    <ul class="dropdown-menu">
      <li>
        <button onclick={handleClickNewProject}>New Project</button>
      </li>
      <li>
        <button onclick={handleClickNewProject}>Open Project</button>
      </li>
      <li>
        <button>New Dialogue</button>
      </li>
      <li>
        <button>Save</button>
      </li>
      <li>
        <button>Save As...</button>
      </li>
      <li>
        <button>Export</button>
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
