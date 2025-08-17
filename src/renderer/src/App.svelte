<script lang="ts">
  import { onMount } from "svelte";
  import { SvelteMap } from "svelte/reactivity";

  import {
    SvelteFlow,
    Background,
    MiniMap,
    Controls,
    MarkerType,
    type Node,
    type Edge,
    type EdgeTypes,
    type NodeTypes,
    type Connection,
    Panel
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import DialogueNode from "./components/node/Dialogue.svelte";
  import PortraitModal from "./components/database/DatabaseModal.svelte";
  import DialogueEdge from "./components/node/DialogueEdge.svelte";
  import BranchContainer from "./components/node/BranchContainer.svelte";
  import Branch from "./components/node/Branch.svelte";
  import SkillCheck from "./components/node/SkillCheck.svelte";
  import StartNode from "./components/node/StartNode.svelte";
  import EndNode from "./components/node/EndNode.svelte";
  import Titlebar from "./components/titlebar/Titlebar.svelte";
  import DialogueSelect from "./components/dialogue-select/DialogueSelect.svelte";
  import ButtonsContainer from "./components/buttons/ButtonsContainer.svelte";

  import { modal } from "./stores/dbModal.svelte";
  import { dialogues } from "./stores/dialogueStore.svelte";
  import {
    loadSpeakersFromDb,
    loadPortraitsFromDb,
    loadSkillsFromDb
  } from "./stores/dbStore.svelte";
  import { nodeButtons } from "./utils/buttons";
  import { MARKER_END_HEIGHT, MARKER_END_WIDTH } from "./utils/utils";
  import type { DialogueNodeType, ConnectedTypeData } from "./utils/types";

  // Alias for parameter type in SvelteFlow's onedgeclick callback.
  type EdgeClickEvent = {
    edge: Edge;
    event: MouseEvent;
  };

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

    const sourceNode = dialogues.nodes.find((n) => n.id === sourceId);
    if (sourceNode.type === "branchContainer") {
      return;
    }

    const data = sourceNode.data as ConnectedTypeData;
    data.next = targetId;
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

    const data = sourceNode.data as ConnectedTypeData;
    data.edges = dialogues.edges.filter((e) => e.id !== edge.id);
  }

  function exportJSON(): void {
    // TODO: Add this functionality to a button eventually.
    const json = [];
    const map: SvelteMap<string, Node> = new SvelteMap();
    let start: Node = null;

    for (const node of dialogues.nodes) {
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
        const branchContainer = top;
        const neighbors = dialogues.nodes.filter((n) => n.parentId === branchContainer.id);

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
        fitView
      >
        <Panel position="top-left">
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
