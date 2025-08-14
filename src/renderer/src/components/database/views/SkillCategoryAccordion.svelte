<script lang="ts">
  import { slide } from "svelte/transition";

  import Checkmark from "../../icons/Checkmark.svelte";
  import DatabaseListButton from "../list/DatabaseListButton.svelte";
  import Chevron from "../../icons/Chevron.svelte";
  import Cross from "../../icons/Cross.svelte";
  import Plus from "../../icons/Plus.svelte";

  import type { SkillCategory, Skill } from "../../../../../shared/types";
  import { addEntity, deleteEntity } from "../../../stores/dbStore.svelte";

  type Props = {
    category: SkillCategory;
  };

  const ICON_SIZE = 24;

  let { category }: Props = $props();
  let open = $state(false);
  let controllerVisible = $state(false);
  let newSkillName = $state("");

  function toggleAccordion(): void {
    open = !open;
  }

  function showController(): void {
    controllerVisible = true;
  }

  function hideController(): void {
    controllerVisible = false;
  }

  function deleteAccordion(): void {
    deleteEntity(category);
  }

  function addSkill(): void {
    if (newSkillName === "") {
      return;
    }

    const newSkill: Skill = {
      id: crypto.randomUUID(),
      kind: "skill",
      category,
      name: newSkillName
    };
    addEntity(newSkill);
    newSkillName = "";
  }
</script>

<div class="accordion">
  <header>
    <button class="toggle-accordion" onclick={toggleAccordion}>
      <Chevron width={ICON_SIZE} height={ICON_SIZE} rotated={open} />
      <span>{category.name}</span>
    </button>
    <button class="delete-accordion" onclick={deleteAccordion}>
      <Cross width={ICON_SIZE} height={ICON_SIZE} />
    </button>
  </header>

  {#if open}
    <ul transition:slide>
      {#each category.skills as skill (skill.id)}
        <DatabaseListButton entity={skill} />
      {/each}
    </ul>

    <div class="add-skill">
      {#if controllerVisible}
        <div class="controller" in:slide>
          <input placeholder="Skill Name" bind:value={newSkillName} />
          <button class="secondary-btn" onclick={addSkill}>
            <Checkmark width={ICON_SIZE / 2} height={ICON_SIZE / 2} />
          </button>
          <button class="secondary-btn" onclick={hideController}>
            <Cross width={ICON_SIZE / 2} height={ICON_SIZE / 2} />
          </button>
        </div>
      {:else}
        <div class="show-controller" in:slide>
          <button class="secondary-btn" onclick={showController}>
            <Plus width={ICON_SIZE / 2} height={ICON_SIZE / 2} />
          </button>
          <span>Add Skill</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  header {
    display: flex;
    gap: 0.5rem;
    justify-content: space-between;
    margin-bottom: 0.8rem;
  }

  .toggle-accordion {
    flex: 1 1 auto;
  }

  .delete-accordion {
    display: flex;
    justify-content: center;
    flex: 0 0 10%;
    border-radius: 0.3rem;
    background-color: var(--secondary-color);
  }

  button {
    display: flex;
    align-items: center;
  }

  .accordion {
    width: 100%;
    padding: 1rem;
    border-radius: 0.5rem;
    box-shadow: var(--node-box-shadow);
  }

  .add-skill {
    margin: 0.8rem 0;
  }

  .controller {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .show-controller {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .show-controller span {
    font-size: 0.8rem;
  }
</style>
