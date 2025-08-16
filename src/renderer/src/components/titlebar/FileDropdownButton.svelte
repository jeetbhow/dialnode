<script lang="ts">
  import { clickOutside } from "../../attachments/attachments";

  let open = $state(false);
</script>

<div class="dropdown">
  <button onclick={() => (open = !open)} {@attach clickOutside(() => (open = false))}>File</button>
  {#if open}
    <ul class="dropdown-menu">
      <li>
        <button>New Project</button>
      </li>
      <li>
        <button>Open</button>
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
    border-radius: 0.5rem;
    min-width: 400px;
    z-index: 1000;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .dropdown-menu li {
    padding: 0.2rem 3rem;
    margin: 0.4rem;
    border-radius: 0.3rem;
  }

  .dropdown-menu li:hover {
    background: #ccc9c9;
  }

  .dropdown-menu li > button {
    width: 100%;
  }
</style>
