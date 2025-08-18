<script lang="ts">
  import { tick } from "svelte";
  import { dialogues } from "../../stores/dialogueStore.svelte";

  let inputRef: HTMLInputElement | null = $state(null);
  let newName = $derived(dialogues.get(dialogues.selectedIndex).name);

  $effect(() => {
    if (dialogues.editing) {
      (async () => {
        await tick();
        inputRef?.focus();
        inputRef?.select();
      })();
    }
  });

  function handleSubmit(event: Event) {
    event.preventDefault();
    dialogues.renameSelected(newName);
    dialogues.editing = false;
  }
</script>

<div class="sidebar">
  <ul class="project-dialogues">
    {#each dialogues.data as dialogue, i (dialogue)}
      <li class:selected={dialogues.selectedIndex === i}>
        {#if dialogues.selectedIndex === i && dialogues.editing}
          <form onsubmit={handleSubmit}>
            <input
              type="text"
              bind:value={newName}
              bind:this={inputRef}
              onblur={() => (dialogues.editing = false)}
            />
          </form>
        {:else}
          <button onclick={() => dialogues.selectDialogue(i)}>
            {dialogue.name}
          </button>
        {/if}
      </li>
    {/each}
  </ul>
</div>

<style>
  .sidebar {
    background-color: #fafafa;
    width: 15%;
    min-width: 250px;
    height: 100%;
  }

  .project-dialogues {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .project-dialogues li:hover {
    background-color: rgb(223, 223, 223);
  }

  .project-dialogues button {
    padding: 0.4rem;
    padding-left: 3rem;
    box-sizing: border-box;
    width: 100%;
    font-size: 0.8rem;
    white-space: pre;
    color: rgb(58, 61, 64);
  }

  .project-dialogues input {
    box-sizing: border-box;
    width: 100%;
    outline: 2px solid var(--primary-color);
    padding: 0.4rem 0;
    padding-left: 3rem;
    background: none;
    font-size: 0.8rem;
  }

  .project-dialogues form {
    margin: 0 0.2rem;
  }

  .selected {
    background-color: rgb(223, 223, 223);
  }
</style>
