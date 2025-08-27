<script lang="ts">
  import { onMount } from "svelte";

  import {
    SvelteFlow,
    Panel,
    MiniMap,
    Controls,
    Background,
    MarkerType,
    type Edge,
    type NodeTypes,
    type EdgeTypes,
    type Connection
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import DialogueNodeEdge from "./components/edges/DialogueNodeEdge.svelte";
  import StartNode from "./components/nodes/boundaries/StartNode.svelte";
  import EndNode from "./components/nodes/boundaries/EndNode.svelte";
  import TextNode from "./components/nodes/text/TextNode.svelte";
  import BranchContainerNode from "./components/nodes/branch/BranchContainerNode.svelte";
  import SkillCheckNode from "./components/nodes/branch/SkillCheckNode.svelte";
  import BranchNode from "./components/nodes/branch/BranchNode.svelte";
  import DialogueSelect from "./components/dialogue/DialogueSelect.svelte";
  import ButtonsContainer from "../shared/components/buttons/ButtonsContainer.svelte";
  import DatabaseModal from "./components/database/DatabaseModal.svelte";

  import { modal } from "./stores/dbModal.svelte";
  import { graph } from "./stores/graphStore.svelte";
  import {
    loadPortraitsFromDb,
    loadSkillsFromDb,
    loadSpeakersFromDb
  } from "./stores/dbStore.svelte";

  import type { ConnectableNodeType, DialogueNode } from "../../../shared/types";
  import { MARKER_END_HEIGHT, MARKER_END_WIDTH } from "./utils/utils";
  import { nodeButtons } from "./utils/buttons";
  import { fetchRepository } from "./stores/repositoryStore.svelte";

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  const nodeTypes: NodeTypes = {
    text: TextNode,
    branchContainer: BranchContainerNode,
    branch: BranchNode,
    skillCheck: SkillCheckNode,
    start: StartNode,
    end: EndNode
  };

  const edgeTypes: EdgeTypes = {
    dialogueEdge: DialogueNodeEdge
  };

  onMount(async () => {
    await fetchRepository();
    await loadSpeakersFromDb();
    await loadPortraitsFromDb();
    await loadSkillsFromDb();
    //await dialogues.loadFromDb();
  });

  function handleNodeDragStop(_: {
    targetNode: DialogueNode<Record<string, unknown>>;
    nodes: DialogueNode<Record<string, unknown>>[];
    event: MouseEvent | TouchEvent;
  }): void {
    //dialogues.save();
  }

  function handleConnect(connection: Connection): void {
    const sourceId = connection.source;
    const targetId = connection.target;

    const sourceNode = graph.nodes.find((n) => n.id === sourceId);
    if (sourceNode.type === "end" || sourceNode.type === "branchContainer") {
      throw Error("Invalid state: source connection on end or branch container node.");
    }

    const connectable = sourceNode as ConnectableNodeType;
    connectable.data.next = targetId;
    //dialogues.save();
  }

  function handleBeforeConnect(connection: Connection): Edge {
    // Prevent connections to self.
    graph.edges = graph.edges.filter(
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
    const sourceNode = graph.nodes.find((n) => n.id === sourceId);

    if (sourceNode.type === "branchContainer" || sourceNode.type === "end") {
      throw new Error("Invalid state: source conection on branchContainer");
    }

    const connectable = sourceNode as ConnectableNodeType;
    connectable.data.next = null;

    graph.edges = graph.edges.filter((e) => e.id !== edge.id);
    //dialogues.save();
  }
</script>

<div class="main">
  <DialogueSelect />
  {#if graph.nodes !== null && graph.edges !== null}
    <SvelteFlow
      bind:nodes={graph.nodes}
      bind:edges={graph.edges}
      {nodeTypes}
      {edgeTypes}
      onconnect={handleConnect}
      onbeforeconnect={handleBeforeConnect}
      onedgeclick={handleEdgeClick}
      onnodedragstop={handleNodeDragStop}
    >
      <Panel position="top-center">
        <ButtonsContainer flexDirection="row" buttons={nodeButtons} />
      </Panel>
      <MiniMap />
      <Controls />
      <Background />
    </SvelteFlow>
  {/if}
</div>

{#if modal.open}
  <DatabaseModal />
{/if}

<style>
  .main {
    display: flex;
    width: 100%;
    height: 100%;
  }
</style>
