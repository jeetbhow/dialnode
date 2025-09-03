<script lang="ts">
  import Cross from "../../../../shared/components/icons/Cross.svelte";

  import { Handle, Position, useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import { useDb } from "../../../stores/dbStore.svelte";
  import type { BranchContainerNodeType, SkillCheckNodeType } from "../../../../../../shared/types";
  import { graph } from "../../../stores/graphStore.svelte";

  const ICON_SIZE = 26;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  const db = useDb();
  const { deleteElements } = useSvelteFlow();

  let { id, data, parentId }: NodeProps<SkillCheckNodeType> = $props();
  let skills = $derived(db.skillCategories.flatMap((category) => category.skills) ?? []);
  let selectedIndex = $state(0);

  function handleSkillChange(e: Event): void {
    const target = e.target as HTMLSelectElement;
    selectedIndex = target.selectedIndex;
    data.skill = skills[selectedIndex];
  }

  function handleDifficultyChange(e: Event): void {
    const target = e.target as HTMLInputElement;

    graph.nodes = graph.nodes.map((n) =>
      n.id === id ? { ...n, data: { ...n.data, difficulty: parseInt(target.value) } } : n
    );

    graph.save();
  }

  function handleTextChange(e: Event): void {
    const target = e.target as HTMLInputElement;

    graph.nodes = graph.nodes.map((n) =>
      n.id === id ? { ...n, data: { ...n.data, text: target.value } } : n
    );

    graph.save();
  }

  function deleteThisNode(): void {
    const branchContainer = graph.nodes.find((n) => n.id === parentId) as BranchContainerNodeType;

    const branches = branchContainer.data.branches;
    branchContainer.data.branches = branches.filter((b) => b !== id);

    deleteElements({ nodes: [{ id }] });
  }
</script>

<Handle
  type="source"
  position={Position.Right}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--source-handle-color);"
/>
<div {id} class="container">
  <div class="content">
    <header>
      <h3>Skill Check</h3>
      <button onclick={deleteThisNode}>
        <Cross width={ICON_SIZE} height={ICON_SIZE} />
      </button>
    </header>
    <div class="text">
      <label for="text">Text:</label>
      <input
        id="text"
        value={data.text}
        spellcheck="false"
        placeholder="Enter skill check text."
        onchange={handleTextChange}
      />
    </div>
    <div class="skill">
      <label for="skill">Skill: </label>
      <select id="skill" onchange={handleSkillChange}>
        {#each skills as skill (skill.id)}
          <option value={skill.name}>{skill.name}</option>
        {/each}
      </select>
    </div>

    <div class="difficulty">
      <label for="difficulty">Difficulty: </label>
      <input
        id="difficulty"
        class="nodrag"
        type="number"
        min="1"
        max="20"
        step="1"
        value={data.difficulty}
        onchange={handleDifficultyChange}
      />
    </div>
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
  }

  header h3 {
    margin: auto;
  }

  select {
    font-family: "Iosevka-Regular";
    padding: 0.5rem;
  }

  .content > div {
    align-items: center;
    gap: 0.5rem;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin: 0.3rem;
  }

  input {
    font-family: "Iosevka-Regular";
  }

  label {
    display: block;
    min-width: 50px;
    font-size: 0.9rem;
    font-weight: 600;
  }
</style>
