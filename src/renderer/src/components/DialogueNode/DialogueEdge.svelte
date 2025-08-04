<script lang="ts">
  import { BaseEdge, getBezierPath, type EdgeProps } from '@xyflow/svelte';

  let { id, sourceX, sourceY, targetX, targetY, markerEnd }: EdgeProps = $props();
  const curvature = 0.4;

  let [path] = $derived(getBezierPath({ sourceX, sourceY, targetX, targetY, curvature }));
</script>

<BaseEdge {id} {path} {markerEnd} />
