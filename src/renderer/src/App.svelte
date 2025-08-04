<script lang="ts">
  import {
    SvelteFlow,
    Background,
    Panel,
    MiniMap,
    Controls,
    MarkerType,
    type Edge,
    type EdgeTypes,
    type NodeTypes,
    type Connection
  } from '@xyflow/svelte';

  import '@xyflow/svelte/dist/style.css';
  import type { Button, DialogueNodeType, Portrait, Speaker } from './types';
  import { modal } from './stores/portraitModal.svelte';

  import ButtonsContainer from './components/buttons/ButtonsContainer.svelte';
  import DialogueNode from './components/dialogueNode/DialogueNode.svelte';
  import PortraitModal from './components/database/DatabaseModal.svelte';

  import DialogueEdge from './components/dialogueNode/DialogueEdge.svelte';

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  let projectDir: string = $state('');
  let portraits: Portrait[] = $state([]);
  let speakers: Speaker[] = $state([]);

  let nodes: DialogueNodeType[] = $state.raw([]);
  let edges: Edge[] = $state.raw([]);

  const nodeTypes: NodeTypes = {
    dialogueNode: DialogueNode
  };

  const edgeTypes: EdgeTypes = {
    dialogueEdge: DialogueEdge
  };

  function handleConnect(connection: Connection): Edge {
    edges = edges.filter(
      (e) => e.source !== connection.source || e.sourceHandle !== connection.sourceHandle
    );
    return {
      id: crypto.randomUUID(),
      ...connection,
      type: 'dialogueEdge',
      markerEnd: {
        type: MarkerType.ArrowClosed,
        width: 28,
        height: 28
      }
    };
  }

  function handleEdgeClick(event: EdgeClickEvent): void {
    const { edge } = event;
    edges = edges.filter((e) => e.id !== edge.id);
  }

  async function selectDirectory(): Promise<void> {
    projectDir = await window.api.selectDirectory();
  }

  const addNode = () => {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 }; // TODO: We're going to be replacing fixed-positions with drag and drop later on.

    const newDialogueNode: DialogueNodeType = {
      id,
      type: 'dialogueNode',
      position,
      data: { text: '', showOptions: false }
    };

    nodes = [...nodes, newDialogueNode];
  };

  const controlButtons: Button[] = [
    {
      text: 'Database',
      onClick: () => {
        modal.open = true;
      }
    },
    {
      text: 'Clear',
      onClick: () => {
        nodes = [];
      }
    },
    {
      text: 'Project',
      onClick: selectDirectory
    }
  ];

  const nodeButtons: Button[] = [
    {
      text: '+ Node',
      onClick: addNode
    }
  ];
</script>

<div style:width="100vw" style:height="100vh">
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    onbeforeconnect={handleConnect}
    onedgeclick={handleEdgeClick}
    fitView
  >
    <MiniMap />
    <Controls />
    <Panel position="top-left">
      <ButtonsContainer flexDirection="row" buttons={nodeButtons} />
    </Panel>
    <Panel position="top-right">
      <ButtonsContainer flexDirection="row" buttons={controlButtons} />
    </Panel>
    <Background />
  </SvelteFlow>
  {#if modal.open}
    <PortraitModal bind:portraits bind:speakers bind:projectDir />
  {/if}
</div>

<style>
  * {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  }
</style>
