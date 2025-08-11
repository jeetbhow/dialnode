<script lang="ts">
  import Cross from "../icons/Cross.svelte";
  import Plus from "../icons/Plus.svelte";
  import Dice from "../icons/Dice.svelte";

  import "../../styles.css";

  import { Handle, NodeResizer, Position, useSvelteFlow, type NodeProps } from "@xyflow/svelte";
  import type { BranchContainerNodeType } from "../../utils/types";
  import { BRANCH_NODE_INITIAL_HEIGHT, BRANCH_NODE_INITIAL_WIDTH } from "../../utils/utils";

  const ICON_SIZE = 24;
  const DEFAULT_HANDLE_STYLE = "width: 0.5rem; height: 0.5rem";

  let { id, data, selected }: NodeProps<BranchContainerNodeType> = $props();
  const { deleteElements } = useSvelteFlow();

  function addBranch() {
    data.addBranch("branch", id);
  }

  function addSkillCheck() {
    data.addSkillCheck("skillCheck", id);
  }
</script>

<NodeResizer
  autoScale={true}
  isVisible={selected}
  minWidth={BRANCH_NODE_INITIAL_WIDTH}
  minHeight={BRANCH_NODE_INITIAL_HEIGHT}
  style="margin: 0.5rem"
/>
<Handle
  type="target"
  position={Position.Left}
  style="{DEFAULT_HANDLE_STYLE}; background-color: var(--target-handle-color);"
/>
<div {id} class="container">
  <div class="control-btns">
    <button aria-label="Add branch." onclick={addBranch}>
      <Plus width={ICON_SIZE} height={ICON_SIZE} />
    </button>
    <button aria-label="Add skill check node" onclick={addSkillCheck}>
      <Dice width={ICON_SIZE} height={ICON_SIZE} />
    </button>
  </div>
  <button aria-label="Delete node" onclick={() => deleteElements({ nodes: [{ id }] })}>
    <Cross />
  </button>
</div>

<style>
  .container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: start;
    justify-content: space-between;
  }

  .control-btns {
    display: flex;
    margin-top: 0.1rem;
    gap: 0.3rem;
  }
</style>
