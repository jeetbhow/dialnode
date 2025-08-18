<script lang="ts">
  import { BaseEdge, getBezierPath, type EdgeProps } from "@xyflow/svelte";

  const CURVATUURE = 0.4;

  let { id, sourceX, sourceY, targetX, targetY, markerEnd }: EdgeProps = $props();

  let [path] = $derived(
    getBezierPath({ sourceX, sourceY, targetX, targetY, curvature: CURVATUURE })
  );
</script>

<BaseEdge {id} {path} {markerEnd} />
