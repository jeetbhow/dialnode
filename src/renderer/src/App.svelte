<script lang="ts">
  import { onMount } from "svelte";
  import ButtonsContainer from "./components/buttons/ButtonsContainer.svelte";
  import DialogueNode from "./components/node/Dialogue.svelte";
  import PortraitModal from "./components/database/DatabaseModal.svelte";
  import DialogueEdge from "./components/node/DialogueEdge.svelte";
  import BranchContainer from "./components/node/BranchContainer.svelte";
  import Branch from "./components/node/Branch.svelte";
  import SkillCheck from "./components/node/SkillCheck.svelte";
  import StartNode from "./components/node/StartNode.svelte";
  import EndNode from "./components/node/EndNode.svelte";
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
  import { SvelteMap } from "svelte/reactivity";
  import "@xyflow/svelte/dist/style.css";
  import { modal } from "./stores/dbModal.svelte";
  import {
    useDb,
    loadSpeakersFromDb,
    loadPortraitsFromDb,
    loadSkillsFromDb
  } from "./stores/dbStore.svelte";
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
    SkillCheckNodeType,
    StartNodeType,
    ConnectedTypeData
  } from "./utils/types";
  import Titlebar from "./components/titlebar/Titlebar.svelte";

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  const db = useDb();

  let nodes = $state.raw<Node[]>([]);
  let edges = $state.raw<Edge[]>([]);

  onMount(() => {
    loadSpeakersFromDb();
    loadPortraitsFromDb();
    loadSkillsFromDb();
  });

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

  function handleConnect(connection: Connection): void {
    const sourceId = connection.source;
    const targetId = connection.target;

    const sourceNode = nodes.find((n) => n.id === sourceId);
    if (sourceNode.type === "branchContainer") {
      return;
    }

    const data = sourceNode.data as ConnectedTypeData;
    data.next = targetId;
  }

  function handleBeforeConnect(connection: Connection): Edge {
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

    const sourceId = edge.source;
    const sourceNode = nodes.find((n) => n.id === sourceId);

    if (sourceNode.type === "branchContainer") {
      alert("A branch container should not trigger an edge click event.");
      return;
    }

    const data = sourceNode.data as ConnectedTypeData;
    data.edges = edges.filter((e) => e.id !== edge.id);
  }

  async function selectDirectory(): Promise<void> {
    setProjectDirectory(await window.api.selectDirectory());
  }

  function addStart(): void {
    const newNode: StartNodeType = {
      id: crypto.randomUUID(),
      type: "start",
      position: { x: 0, y: 0 },
      data: { next: "" }
    };

    nodes = [...nodes, newNode];
  }

  function addEnd(): void {
    const newNode = {
      id: crypto.randomUUID(),
      type: "end",
      position: { x: 0, y: 0 },
      data: {}
    };

    nodes = [...nodes, newNode];
  }

  function addDialogue(): void {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 }; // TODO: Replacing fixed-positions with drag and drop later on.

    const newNode: DialogueNodeType = {
      id,
      type: "dialogue",
      position,
      data: { text: "", showOptions: false, next: null }
    };

    nodes = [...nodes, newNode];
  }

  function addBranchContainer(): void {
    const id = crypto.randomUUID();
    const newNode: BranchContainerNodeType = {
      id,
      type: "branchContainer",
      position: { x: 0, y: 0 },
      data: {
        addBranch: () => addBranch(id),
        addSkillCheck: () => addSkillCheck(id)
      },
      width: BRANCH_NODE_INITIAL_WIDTH,
      height: BRANCH_NODE_INITIAL_HEIGHT
    };

    nodes = [...nodes, newNode];
  }

  function addBranch(parentId: string): void {
    const newNode: BranchNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "branch",
      position: { x: 0, y: 0 },
      data: { name: "", next: "" }
    };

    nodes = [...nodes, newNode];
  }

  function addSkillCheck(parentId: string): void {
    const newNode: SkillCheckNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "skillCheck",
      position: { x: 0, y: 0 },
      data: { skill: db.skillCategories[0].skills[0], difficulty: 0, next: "" }
    };

    nodes = [...nodes, newNode];
  }

  function exportJSON(): void {
    const json = [];
    const map: SvelteMap<string, Node> = new SvelteMap();
    let start: Node = null;

    for (const node of nodes) {
      map.set(node.id, node);
      if (node.type === "start") {
        start = node;
      }
    }

    if (start === null) {
      alert("Could not find the start node for the dialogue.");
      return;
    }

    const stack = [start];
    while (stack.length !== 0) {
      const top = stack.pop();

      if (top === undefined) {
        continue;
      }

      if (top.type === "branchContainer") {
        const branchContainer = top as BranchContainerNodeType;
        const neighbors = nodes.filter((n) => n.parentId === branchContainer.id);

        const entry = {
          id: branchContainer.id,
          type: "branch",
          next: []
        };

        for (const neighbor of neighbors) {
          entry.next.push(neighbor.id);
        }

        json.push(entry);
      } else if (top.type === "dialogue") {
        const dialogue = top as DialogueNodeType;
        json.push({
          id: dialogue.id,
          type: "text",
          speaker: dialogue.data.speaker?.name ?? null,
          portrait: dialogue.data.portrait?.virtualPath ?? null,
          next: dialogue.data.next
        });
        stack.push(map.get(dialogue.data.next));
      } else {
        const data = top.data as ConnectedTypeData;
        json.push({
          id: top.id,
          type: top.type
        });
        stack.push(map.get(data.next));
      }
    }

    window.api.exportJson(json);
  }

  function clearGraph(): void {
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
    { text: "Project", onClick: selectDirectory },
    { text: "Export JSON", onClick: exportJSON }
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

<div class="app">
  <Titlebar />
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    onconnect={handleConnect}
    onbeforeconnect={handleBeforeConnect}
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
    <PortraitModal />
  {/if}
</div>

<style>
  :global(body) {
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    margin: 0;
    padding: 0;
  }

  .app {
    width: 100vw;
    height: 100vh;
  }
</style>
