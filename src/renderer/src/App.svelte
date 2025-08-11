<script lang="ts">
  import ButtonsContainer from "./components/buttons/ButtonsContainer.svelte";
  import DialogueNode from "./components/node/Dialogue.svelte";
  import PortraitModal from "./components/database/DatabaseModal.svelte";
  import DialogueEdge from "./components/node/DialogueEdge.svelte";
  import BranchContainer from "./components/node/BranchContainer.svelte";
  import Branch from "./components/node/Branch.svelte";
  import SkillCheck from "./components/node/SkillCheck.svelte";
  import StartNode from "./components/node/StartNode.svelte";
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
    BranchNodeType,
    SkillCheckNodeType
  } from "./utils/types";
  import EndNode from "./components/node/EndNode.svelte";

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
    branch: Branch,
    skillCheck: SkillCheck,
    start: StartNode,
    end: EndNode
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

  function addStart() {
    const newNode = {
      id: crypto.randomUUID(),
      type: "start",
      position: { x: 0, y: 0 },
      data: {}
    };

    nodes = [...nodes, newNode];
  }

  function addEnd() {
    const newNode = {
      id: crypto.randomUUID(),
      type: "end",
      position: { x: 0, y: 0 },
      data: {}
    };

    nodes = [...nodes, newNode];
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
      data: { addBranch: () => addBranch(id), addSkillCheck: () => addSkillCheck(id) },
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

  function addSkillCheck(parentId: string) {
    const newNode: SkillCheckNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "skillCheck",
      position: { x: 0, y: 0 },
      data: { skill: null, difficulty: 0 }
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
      text: "+ Start",
      onClick: addStart
    },
    {
      text: "+ Node",
      onClick: addDialogue
    },
    {
      text: "+ Branch",
      onClick: addBranchContainer
    },
    {
      text: "+ End",
      onClick: addEnd
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
