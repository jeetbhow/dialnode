<script lang="ts">
  import { modal } from "../../../../stores/dbModal.svelte";

  type Props = {
    selected: string;
  };

  let { selected = $bindable() }: Props = $props();

  function handleSelect(name: string): void {
    selected = name;
  }
</script>

<div>
  <button
    disabled={modal.request?.type !== "portrait"}
    class:selected={selected === "portrait"}
    onclick={() => handleSelect("portrait")}>Portrait</button
  >
  <button
    disabled={modal.request.type !== "speaker"}
    class:selected={selected === "speaker"}
    onclick={() => handleSelect("speaker")}>Speaker</button
  >
  <button
    disabled={modal.request?.type === "skill"}
    class:selected={selected === "skill"}
    onclick={() => handleSelect("skill")}>Skill</button
  >
</div>

<style>
  button {
    margin-right: 0.5rem;
    padding: 0.3rem;
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-color);
  }

  .selected {
    color: var(--primary-color);
    border-bottom: 1px solid var(--tab-bottom-border-color);
  }

  button[disabled] {
    color: var(--tab-disabled-color);
    cursor: default;
  }
</style>
