<script lang="ts">
  import Cross from "../icons/Cross.svelte";
  import Maximize from "../icons/Maximize.svelte";
  import Minimize from "../icons/Minimize.svelte";

  const CROSS_SIZE = 24;
  const MAXIMIZE_ICON_SIZE = 16;
  const MINIMIZE_ICON_SIZE = 16;
</script>

<ul>
  <button class="minimize" onclick={() => window.api.minimize()}>
    <Minimize width={MINIMIZE_ICON_SIZE} height={MINIMIZE_ICON_SIZE} />
  </button>
  <button class="maximize" onclick={() => window.api.maximize()}>
    <Maximize width={MAXIMIZE_ICON_SIZE} height={MAXIMIZE_ICON_SIZE} />
  </button>
  <button class="close" onclick={() => window.api.close()}>
    <Cross width={CROSS_SIZE} height={CROSS_SIZE} />
  </button>
</ul>

<style>
  ul {
    all: unset;
    display: flex;
    justify-content: end;
    -webkit-app-region: drag;
  }

  button {
    padding: 0.4rem 0.9rem;
    transition: background-color 100ms ease-in-out;
    -webkit-app-region: no-drag;
  }

  .close:hover {
    background-color: #e64242;
  }

  .maximize:hover,
  .minimize:hover {
    background-color: #d4d2d2;
  }
</style>
