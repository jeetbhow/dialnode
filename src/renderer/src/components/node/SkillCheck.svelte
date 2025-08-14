<script lang="ts">
  import Cross from "../icons/Cross.svelte";

  import { Handle, Position, useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import { useDb } from "../../stores/dbStore.svelte";
  import type { SkillCheckNodeType } from "../../utils/types";

  const ICON_SIZE = 16;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  const db = useDb();
  const { deleteElements } = useSvelteFlow();

  let { id, data }: NodeProps<SkillCheckNodeType> = $props();
  let skills = $derived(db.skillCategories.flatMap((category) => category.skills) ?? []);
  let selectedIndex = $state(0);

  function handleSkillChange(e: Event): void {
    const target = e.target as HTMLSelectElement;
    selectedIndex = target.selectedIndex;
    data.skill = skills[selectedIndex];
  }

  function handleDifficultyChange(e: Event): void {
    const target = e.target as HTMLInputElement;
    data.difficulty = parseInt(target.value);
  }

  function deleteThisNode(): void {
    deleteElements({ nodes: [{ id }] });
  }
</script>

<Handle
  type="source"
  position={Position.Right}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
/>
<div {id} class="container">
  <header>
    <p>Skill Check</p>
    <button onclick={deleteThisNode}>
      <Cross width={ICON_SIZE} height={ICON_SIZE} />
    </button>
  </header>
  <div>
    <label for="skills">Skill: </label>
    <select id="skills" onchange={handleSkillChange}>
      {#each skills as skill (skill.id)}
        <option value={skill.name}>{skill.name}</option>
      {/each}
    </select>
  </div>

  <div>
    <label for="difficulty">Difficulty: </label>
    <input
      id="difficulty"
      class="nodrag"
      type="number"
      min="1"
      max="20"
      step="1"
      onchange={handleDifficultyChange}
    />
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    font-size: 0.5rem;
  }

  select {
    font-family: "Iosevka-Regular";
    padding: 0.5rem;
  }

  .container {
    font-size: 0.7rem;
  }

  .container > div {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  input {
    font-family: "Iosevka-Regular";
  }

  label {
    min-width: 50px;
  }
</style>
