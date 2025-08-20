<script lang="ts">
  import { dialogues } from "../../../stores/dialogueStore.svelte";
  import Cross from "../icons/Cross.svelte";
  import Maximize from "../icons/Maximize.svelte";
  import Minimize from "../icons/Minimize.svelte";
  import EditDropDown from "./EditDropDown.svelte";
  import FileDropdown from "./FileDropdown.svelte";

  const CROSS_SIZE = 24;
  const MAXIMIZE_ICON_SIZE = 16;
  const MINIMIZE_ICON_SIZE = 16;

  type Props = {
    view: string;
  };

  let { view }: Props = $props();

  function handleClose(): void {
    dialogues.saveToDb();
    window.api.close();
  }
</script>

<div class="titlebar">
  <div class="menu">
    {#if view === "main"}
      <FileDropdown />
      <EditDropDown />
    {/if}
  </div>

  <div class="window-controls">
    <button class="minimize" onclick={() => window.api.minimize()}>
      <Minimize width={MINIMIZE_ICON_SIZE} height={MINIMIZE_ICON_SIZE} />
    </button>
    <button class="maximize" onclick={() => window.api.maximize()}>
      <Maximize width={MAXIMIZE_ICON_SIZE} height={MAXIMIZE_ICON_SIZE} />
    </button>
    <button class="close" onclick={handleClose}>
      <Cross width={CROSS_SIZE} height={CROSS_SIZE} />
    </button>
  </div>
</div>

<style>
  .titlebar {
    display: flex;
    justify-content: space-between;
    background-color: #f6f7fa;
    -webkit-app-region: drag;
  }

  .titlebar > div {
    display: flex;
  }

  .menu {
    align-items: center;
    margin-left: 1rem;
  }

  button {
    padding: 0.4rem 0.9rem;
    transition: background-color 100ms ease-in-out;
  }

  .close:hover {
    background-color: #e64242;
  }

  .maximize:hover,
  .minimize:hover {
    background-color: #d4d2d2;
  }
</style>
