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
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import { modal } from "./stores/dbModal.svelte";
  import { setProjectDirectory } from "./stores/projectStore.svelte";
  import {
    BRANCH_NODE_INITIAL_WIDTH,
    BRANCH_NODE_INITIAL_HEIGHT,
    MARKER_END_HEIGHT,
    MARKER_END_WIDTH
  } from "./utils/utils";
  import type {
    DialogueNodeType,
    Button,
    BranchContainerNodeType,
    BranchNodeType
  } from "./utils/types";

  import ButtonsContainer from "./components/buttons/ButtonsContainer.svelte";
  import DialogueNode from "./components/node/Dialogue.svelte";
  import PortraitModal from "./components/database/DatabaseModal.svelte";
  import DialogueEdge from "./components/node/DialogueEdge.svelte";
  import BranchContainer from "./components/node/BranchContainer.svelte";
  import Branch from "./components/node/Branch.svelte";

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  let nodes = $state.raw<Node[]>([]);
  let edges = $state.raw<Edge[]>([]);

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
      type: "dialogueEdge",
      markerEnd: {
        type: MarkerType.ArrowClosed,
        width: MARKER_END_WIDTH,
        height: MARKER_END_HEIGHT
      }
    };
  }

  function handleEdgeClick(event: EdgeClickEvent): void {
    const { edge } = event;
    edges = edges.filter((e) => e.id !== edge.id);
  }

  async function selectDirectory(): Promise<void> {
    setProjectDirectory(await window.api.selectDirectory());
  }

  function addDialogue() {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 }; // TODO: Replacing fixed-positions with drag and drop later on.

    const newNode: DialogueNodeType = {
      id,
      type: "dialogue",
      position,
      data: { text: "", showOptions: false }
    };

    nodes = [...nodes, newNode];
  }

  function addBranchContainer() {
    const id = crypto.randomUUID();
    const newNode: BranchContainerNodeType = {
      id,
      type: "branchContainer",
      position: { x: 0, y: 0 },
      data: { addBranch: () => addBranch(id) },
      width: BRANCH_NODE_INITIAL_WIDTH,
      height: BRANCH_NODE_INITIAL_HEIGHT
    };

    nodes = [...nodes, newNode];
  }

  function addBranch(parentId: string) {
    const newNode: BranchNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "branch",
      position: { x: 0, y: 0 },
      data: { name: "" }
    };

    nodes = [...nodes, newNode];
  }

  function clearGraph() {
    nodes = [];
    edges = [];
  }

  const controlButtons: Button[] = [
    {
      text: "Database",
      onClick: () => {
        modal.open = true;
      }
    },
    { text: "Clear", onClick: clearGraph },
    { text: "Project", onClick: selectDirectory }
  ];

  const nodeButtons: Button[] = [
    {
      text: "+ Node",
      onClick: addDialogue
    },
    {
      text: "+ Branch",
      onClick: addBranchContainer
    }
  ];
</script>

<div>
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    onbeforeconnect={handleConnect}
    onedgeclick={handleEdgeClick}
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
    <PortraitModal />
  {/if}
</div>

<style>
  :global(body) {
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    margin: 0;
    padding: 0;
  }

  div {
    width: 100vw;
    height: 100vh;
  }
</style>
