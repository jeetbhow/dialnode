<script lang="ts">
  import { type DialogueNodeType } from "../../../../../shared/types";

  type Props = {
    text: string;
    nodeType?: DialogueNodeType;
  };

  const { text, nodeType }: Props = $props();

  function handleDragStart(event: DragEvent): void {
    if (event.dataTransfer) {
      event.dataTransfer.setData("text/plain", nodeType);
      event.dataTransfer.effectAllowed = "move";
    }
  }

  function handleDragEnd(event: DragEvent): void {
    if (event.dataTransfer) {
      event.dataTransfer.clearData();
    }
  }
</script>

<!-- TODO: fix accessibility issues -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="primary-btn"
  draggable="true"
  ondragstart={handleDragStart}
  ondragend={handleDragEnd}
  role="button"
  tabindex={0}
>
  {text}
</div>

<style>
  div {
    cursor: grab;
  }
</style>
