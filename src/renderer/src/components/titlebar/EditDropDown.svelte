<script lang="ts">
  import { clickOutside } from "../../attachments/attachments";
  import { dialogues } from "../../stores/dialogueStore.svelte";

  let open = $state(false);
</script>

<div class="dropdown">
  <button onclick={() => (open = !open)} {@attach clickOutside(() => (open = false))}>Edit</button>
  {#if open}
    <ul class="dropdown-menu">
      <li>
        <button onclick={() => dialogues.add()}>Add Dialogue</button>
      </li>
      <li>
        <button onclick={() => (dialogues.editing = true)}>Rename Dialogue</button>
      </li>
      <li>
        <button onclick={() => dialogues.removeSelected()}>Remove Dialogue</button>
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
