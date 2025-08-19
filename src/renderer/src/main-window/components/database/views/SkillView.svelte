<script lang="ts">
  import Plus from "../../../../shared/components/icons/Plus.svelte";
  import Checkmark from "../../../../shared/components/icons/Checkmark.svelte";
  import SkillCategoryAccordion from "./SkillCategoryAccordion.svelte";

  import "./views.css";
  import type { SkillCategory } from "../../../../../../shared/types";

  import { slide } from "svelte/transition";
  import { useDb, addEntity } from "../../../../stores/dbStore.svelte";
  import Cross from "../../../../shared/components/icons/Cross.svelte";

  const ICON_SIZE = 24;

  const db = useDb();

  let newSkillCategoryName = $state("");
  let controllerOpen = $state(false);

  function openController(): void {
    controllerOpen = true;
  }

  function closeController(): void {
    controllerOpen = false;
  }

  function addSkillCategory(): void {
    if (newSkillCategoryName === "") {
      return;
    }

    const newSkillCategory: SkillCategory = {
      id: crypto.randomUUID(),
      kind: "skillCategory",
      name: newSkillCategoryName,
      skills: []
    };
    addEntity(newSkillCategory);
    newSkillCategoryName = "";
    controllerOpen = false;
  }
</script>

<div class="parent">
  <div class="view">
    <ul>
      {#each db.skillCategories as category (category.id)}
        <SkillCategoryAccordion {category} />
      {/each}
    </ul>
  </div>

  <div class="add-skill-category">
    {#if controllerOpen}
      <div class="add-skill-category-controller" in:slide>
        <input bind:value={newSkillCategoryName} placeholder="Skill Category Name" />
        <button class="secondary-btn" onclick={addSkillCategory}>
          <Checkmark width={ICON_SIZE} height={ICON_SIZE} />
        </button>
        <button class="secondary-btn" onclick={closeController}>
          <Cross width={ICON_SIZE} height={ICON_SIZE} />
        </button>
      </div>
    {:else}
      <button class="primary-btn" onclick={openController} in:slide>
        <Plus width={24} height={24} />
      </button>
      <span>Add Skill Category</span>
    {/if}
  </div>
</div>

<style>
  button {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  ul {
    all: unset;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
  }

  .parent {
    display: flex;
    flex-direction: column;
    justify-content: start;
    margin: 0 3rem;
  }

  .add-skill-category {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.9rem;
    margin-left: 2.2rem;
  }

  .add-skill-category-controller {
    display: flex;
    gap: 0.3rem;
  }
</style>
