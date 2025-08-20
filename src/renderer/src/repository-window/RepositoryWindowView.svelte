<script lang="ts">
  import { fly } from "svelte/transition";

  import logo from "../assets/logo.png";
  import { APP_VERSION, FORMAT_VERSION } from "../shared/utils/appVariables";
  import BackArrow from "../shared/components/icons/BackArrow.svelte";
  import type { Repository } from "../../../shared/types";
  import { setRepository } from "../stores/repositoryStore.svelte";

  const BACK_ARROW_SIZE = 18;
  const DEFAULT_REPOSITORY_NAME = ".dialnode";

  let view = $state<"main" | "create-project">("main");
  let name = $state(DEFAULT_REPOSITORY_NAME);
  let customRepositoryLocation = $state<string | null>(null);
  let godotProjectLocation = $state<string | null>(null);
  let useCustomLocation = $state(false);
  let createButtonEnabled = $derived(
    godotProjectLocation !== null || customRepositoryLocation !== null
  );

  async function handleClickRepositoryLocation() {
    const result = await window.api.selectDirectory({
      title: "Custom Repository",
      buttonLabel: "Select Repository Location",
      properties: ["createDirectory", "promptToCreate"]
    });

    customRepositoryLocation = result ?? customRepositoryLocation;
  }

  async function handleClickGodotProjectDirectory() {
    try {
      const result = await window.api.selectDirectory(
        {
          title: "Godot Project Location",
          buttonLabel: "Select Godot Project Location"
        },
        { godot: true }
      );
      godotProjectLocation = result ?? godotProjectLocation;
    } catch (error) {
      alert("Failed to locate the project.godot file at the root of the directory.");
    }
  }

  function handleCustomLocationCheck() {
    godotProjectLocation = null;
    customRepositoryLocation = null;
  }

  function handleClickBack() {
    view = "main";
    name = DEFAULT_REPOSITORY_NAME;
    useCustomLocation = false;
    godotProjectLocation = null;
    customRepositoryLocation = null;
  }

  async function handleClickCreate() {
    const repository: Repository = {
      id: crypto.randomUUID(),
      name: name,
      location: godotProjectLocation,
      godotProjectLocation: godotProjectLocation,
      createdOn: new Date().toLocaleString(),
      appVersion: APP_VERSION,
      formatVersion: FORMAT_VERSION
    };

    try {
      await window.api.createRepository(repository);
      setRepository(repository);
    } catch (error) {
      alert("Failed to create repository.");
    }
  }
</script>

<div class="project-window-view">
  <img src={logo} alt="logo" width="250" height="200" />
  <p class="version">Version: {APP_VERSION}</p>
  <div class="view">
    {#if view === "create-project"}
      <div class="create-project-view" in:fly={{ x: -200, opacity: 1, duration: 150 }}>
        <header>
          <button class="back-btn" onclick={handleClickBack}>
            <BackArrow width={BACK_ARROW_SIZE} height={BACK_ARROW_SIZE} />
            Back
          </button>
          <h4>Create a Repository</h4>
        </header>
        <div class="menu">
          <div class="view-item">
            <div class="project-location">
              <h5>Name</h5>
              {#if customRepositoryLocation === null}
                <p>Set a custom name for the repository.</p>
              {:else}
                <p>{customRepositoryLocation}</p>
              {/if}
            </div>
            <input bind:value={name} spellcheck="false" />
          </div>
          <div class="view-item {useCustomLocation ? 'disabled' : ''}">
            <div class="godot-project-directory">
              <h5>Godot Project Location</h5>
              {#if godotProjectLocation === null}
                <p>
                  Will also be the location of your repository unless you're using a custom one.
                </p>
              {:else}
                <p>{godotProjectLocation}</p>
              {/if}
            </div>
            <button
              class="secondary-btn"
              onclick={handleClickGodotProjectDirectory}
              disabled={useCustomLocation}>Browse</button
            >
          </div>
          <div class="view-item {useCustomLocation ? '' : 'disabled'}">
            <div class="custom-project-location">
              <header>
                <input
                  type="checkbox"
                  bind:checked={useCustomLocation}
                  onchange={handleCustomLocationCheck}
                />
                <h5>Custom Location</h5>
              </header>
              {#if customRepositoryLocation === null}
                <p>In case you don't have a Godot project. Some features will be inaccessible.</p>
              {:else}
                <p>{customRepositoryLocation}</p>
              {/if}
            </div>
            <button
              class="secondary-btn"
              onclick={handleClickRepositoryLocation}
              disabled={!useCustomLocation}>Browse</button
            >
          </div>
        </div>
        <button
          class="primary-btn {createButtonEnabled ? '' : 'disabled'}"
          disabled={!createButtonEnabled}
          onclick={handleClickCreate}
        >
          Create
        </button>
      </div>
    {:else}
      <div class="main-view">
        <div class="menu" in:fly={{ x: 200, opacity: 1, duration: 150 }}>
          <div class="view-item">
            <div>
              <h5>Create a new repository</h5>
              <p>Create a new local Dialnode repository.</p>
            </div>
            <button class="primary-btn" onclick={() => (view = "create-project")}>Create</button>
          </div>
          <div class="view-item">
            <div>
              <h5>Open a repository</h5>
              <p>Open an already existing local Dialnode repository.</p>
            </div>
            <button class="secondary-btn">Open</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  h4,
  h5,
  p {
    margin: 0;
    padding: 0;
  }

  input {
    font-size: 0.9rem;
  }

  .disabled h5,
  .disabled p {
    color: #989494 !important;
  }

  button.disabled {
    color: rgb(175, 169, 169) !important;
    background-color: rgb(221, 221, 221) !important;
    cursor: default;
  }

  .disabled button {
    color: rgb(175, 169, 169) !important;
    background-color: rgb(221, 221, 221) !important;
    cursor: default;
  }

  .project-window-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    box-sizing: border-box;
    height: 100vh;
  }

  .version {
    font-size: 0.8rem;
    color: #48484a;
  }

  .create-project-view {
    display: flex;
    flex-direction: column;
    margin: 0 8rem;
  }

  .create-project-view header {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .create-project-view header button {
    color: #5a5a5a;
  }

  .create-project-view header button:hover {
    color: #979797;
  }

  .create-project-view h4 {
    font-weight: 500;
    border-bottom: 1px solid #e2dfdf;
    padding-bottom: 0.5rem;
    margin-bottom: 2rem;
  }

  .create-project-view .primary-btn {
    margin: auto;
  }

  .custom-project-location header {
    display: flex;
    flex-direction: row;
  }

  .view {
    margin: auto 0;
  }

  .menu {
    display: flex;
    flex-direction: column;
    margin-bottom: 1rem;
    gap: 1rem;
  }

  .view-item {
    display: flex;
    gap: 10rem;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #e2dfdf;
  }

  .view-item > div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .view-item h5 {
    font-size: 1rem;
    font-weight: 400;
  }

  .view-item p {
    font-size: 0.8rem;
    color: #48484a;
  }

  button {
    min-width: 80px;
  }
</style>
