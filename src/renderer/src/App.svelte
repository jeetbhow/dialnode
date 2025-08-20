<script lang="ts">
  import Titlebar from "./shared/components/titlebar/Titlebar.svelte";
  import MainWindowView from "./main-window/MainWindowView.svelte";
  import RepositoryWindowView from "./repository-window/RepositoryWindowView.svelte";

  import "./styles.css";

  type Props = {
    view: string;
  };

  let { view }: Props = $props();
</script>

<div class="app">
  <Titlebar {view} />
  {#if view === "project"}
    <RepositoryWindowView />
  {:else}
    <MainWindowView />
  {/if}
</div>

<style>
  :global(body) {
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    margin: 0;
    padding: 0;
  }

  .app {
    width: 100vw;
    height: 100vh;
  }
</style>
