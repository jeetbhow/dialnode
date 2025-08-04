<script lang="ts">
  import {
    SvelteFlow,
    Background,
    Panel,
    MiniMap,
    Controls,
    MarkerType,
    type Node,
    type Edge,
    type EdgeTypes,
    type NodeTypes,
    type Connection
  } from '@xyflow/svelte';

  import '@xyflow/svelte/dist/style.css';
  import type { Button, Portrait, Speaker } from './types';
  import { modal } from './stores/portraitModal.svelte';

  import ButtonsContainer from './components/buttons/ButtonsContainer.svelte';
  import DialogueNode from './components/node/Dialogue.svelte';
  import PortraitModal from './components/database/DatabaseModal.svelte';
  import DialogueEdge from './components/node/DialogueEdge.svelte';
  import BranchContainer from './components/node/BranchContainer.svelte';
  import Branch from './components/node/Branch.svelte';

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  const BRANCH_NODE_INITIAL_WIDTH = 240;
  const BRANCH_NODE_INITIAL_HEIGHT = 240;

  let projectDir: string = $state('');
  let portraits: Portrait[] = $state([]);
  let speakers: Speaker[] = $state([]);

  let nodes: Node[] = $state.raw([]);
  let edges: Edge[] = $state.raw([]);

  const nodeTypes: NodeTypes = {
    dialogue: DialogueNode,
    branchContainer: BranchContainer,
    branch: Branch
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

  function addNode(type: string, parentId?: string) {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 }; // TODO: We're going to be replacing fixed-positions with drag and drop later on.

    let newNode: Node = {
      id,
      parentId,
      type: type,
      position,
      data: {},
      ...(parentId && {
        extent: 'parent'
      })
    };

    switch (type) {
      case 'dialogue':
        newNode.data = { text: '', showOptions: false };
        break;
      case 'branchContainer':
        newNode.data = { addBranch: addNode };
        newNode.width = BRANCH_NODE_INITIAL_WIDTH;
        newNode.height = BRANCH_NODE_INITIAL_HEIGHT;
        break;
      case 'branch':
        newNode.data = { name: '' };
      default:
        break;
    }

    nodes = [...nodes, newNode];
  }

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
      onClick: () => addNode('dialogue')
    },
    {
      text: '+ Branch',
      onClick: () => addNode('branchContainer')
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
