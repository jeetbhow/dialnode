<script lang="ts">
  import { onMount } from "svelte";

  import {
    SvelteFlow,
    Background,
    MiniMap,
    Controls,
    MarkerType,
    type Edge,
    type EdgeTypes,
    type NodeTypes,
    type Connection,
    Panel
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import TextNode from "./main-window/components/nodes/text/TextNode.svelte";
  import PortraitModal from "./main-window/components/database/DatabaseModal.svelte";
  import DialogueNodeEdge from "./main-window/components/edges/DialogueNodeEdge.svelte";
  import BranchContainerNode from "./main-window/components/nodes/branch/BranchContainerNode.svelte";
  import BranchNode from "./main-window/components/nodes/branch/BranchNode.svelte";
  import SkillCheckNode from "./main-window/components/nodes/branch/SkillCheckNode.svelte";
  import StartNode from "./main-window/components/nodes/boundaries/StartNode.svelte";
  import EndNode from "./main-window/components/nodes/boundaries/EndNode.svelte";
  import Titlebar from "./shared/components/titlebar/Titlebar.svelte";
  import DialogueSelect from "./main-window/components/dialogue-select/DialogueSelect.svelte";
  import ButtonsContainer from "./shared/components/buttons/ButtonsContainer.svelte";

  import "./styles.css"

  import { modal } from "./stores/dbModal.svelte";
  import { dialogues } from "./stores/dialogueStore.svelte";
  import {
    loadSpeakersFromDb,
    loadPortraitsFromDb,
    loadSkillsFromDb
  } from "./stores/dbStore.svelte";
  import { nodeButtons } from "./utils/buttons";
  import { MARKER_END_HEIGHT, MARKER_END_WIDTH } from "./utils/utils";
  import type { DialogueNode, DialogueNodeData } from "../../shared/types";

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

  onMount(async () => {
    await loadSpeakersFromDb();
    await loadPortraitsFromDb();
    await loadSkillsFromDb();
    await dialogues.loadFromDb();
  });

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

  function handleNodeDragStop(_: {
    targetNode: DialogueNode<DialogueNodeData>;
    nodes: DialogueNode<DialogueNodeData>[];
    event: MouseEvent | TouchEvent;
  }) {
    // You can access the pointer event with event.event
    dialogues.saveSelectedDialogue();
  }

  function handleConnect(connection: Connection): void {
    const sourceId = connection.source;
    const targetId = connection.target;

    const sourceNode = dialogues.nodes.find((n) => n.id === sourceId);
    if (sourceNode.type === "branchContainer") {
      return;
    }

    const data = sourceNode.data as DialogueNodeData;
    data.next = targetId;
    dialogues.saveSelectedDialogue();
  }

  function handleBeforeConnect(connection: Connection): Edge {
    dialogues.edges = dialogues.edges.filter(
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
    const sourceNode = dialogues.nodes.find((n) => n.id === sourceId);

    if (sourceNode.type === "branchContainer") {
      alert("A branch container should not trigger an edge click event.");
      return;
    }

    const data = sourceNode.data as DialogueNodeData;
    data.edges = dialogues.edges.filter((e) => e.id !== edge.id);
  }

  // function exportJSON(): void {
  //   // TODO: Add this functionality to a button eventually.
  //   const json = [];
  //   const map: SvelteMap<string, Node> = new SvelteMap();
  //   let start: Node = null;

  //   for (const node of dialogues.nodes) {
  //     map.set(node.id, node);
  //     if (node.type === "start") {
  //       start = node;
  //     }
  //   }

  //   if (start === null) {
  //     alert("Could not find the start node for the dialogue.");
  //     return;
  //   }

  //   const stack = [start];
  //   while (stack.length !== 0) {
  //     const top = stack.pop();

  //     if (top === undefined) {
  //       continue;
  //     }

  //     if (top.type === "branchContainer") {
  //       const branchContainer = top;
  //       const neighbors = dialogues.nodes.filter((n) => n.parentId === branchContainer.id);

  //       const entry = {
  //         id: branchContainer.id,
  //         type: "branch",
  //         next: []
  //       };

  //       for (const neighbor of neighbors) {
  //         entry.next.push(neighbor.id);
  //       }

  //       json.push(entry);
  //     } else if (top.type === "text") {
  //       const dialogue = top as TextNodeType;
  //       json.push({
  //         id: dialogue.id,
  //         type: "text",
  //         speaker: dialogue.data.speaker?.name ?? null,
  //         portrait: dialogue.data.portrait?.virtualPath ?? null,
  //         next: dialogue.data.next
  //       });
  //       stack.push(map.get(dialogue.data.next));
  //     } else {
  //       const data = top.data as DialogueNodeData;
  //       json.push({
  //         id: top.id,
  //         type: top.type
  //       });
  //       stack.push(map.get(data.next));
  //     }
  //   }

  //   window.api.exportJson(json);
  // }
</script>

<div class="app">
  <Titlebar />
  <div class="main">
    <DialogueSelect />
    {#if dialogues.selectedIndex != null}
      <SvelteFlow
        bind:nodes={dialogues.nodes}
        bind:edges={dialogues.edges}
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

  .main {
    display: flex;
    width: 100%;
    height: 100%;
  }
</style>
