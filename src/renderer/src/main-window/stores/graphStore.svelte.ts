import type { Edge } from "@xyflow/svelte";

import { useDb } from "./dbStore.svelte";
import { BRANCH_NODE_INITIAL_WIDTH, BRANCH_NODE_INITIAL_HEIGHT } from "../utils/utils";

import type {
  Dialogue,
  DialogueNode,
  StartNodeType,
  TextNodeType,
  BranchNodeType,
  SkillCheckNodeType,
  EndNodeType,
  BranchContainerNodeType,
  DialogueNodeType
} from "../../../../shared/types";

const db = useDb();

class Dialogues {
  public nodes = $state.raw<DialogueNode<Record<string, unknown>>[] | null>(null);
  public edges = $state.raw<Edge[] | null>(null);

  public display(dialogue: Dialogue) {
    this.nodes = dialogue.nodes;
    this.edges = dialogue.edges;
  }

  // public async loadFromDb(): Promise<void> {
  //   const serializedDialogue = await window.api.getAllDialogues();
  //   const dialogues: Dialogue[] = serializedDialogue.map((sd) => ({
  //     ...sd,
  //     nodes: sd.nodes.map((node) => ({
  //       id: node.id,
  //       parentId: node.parentId ?? undefined,
  //       extent: node.extent ?? undefined,
  //       type: node.type,
  //       width: node.width ?? undefined,
  //       height: node.height ?? undefined,
  //       position: { x: node.positionX ?? 0, y: node.positionY ?? 0 },
  //       data: JSON.parse(node.data)
  //     }))
  //   }));
  //   this._data = dialogues;
  // }

  // public async saveToDb(): Promise<void> {
  //   this.save();
  //   await window.api.saveDialogues(this.serialize());
  // }

  // public save(): void {
  //   if (this._selectedIndex === null) {
  //     return;
  //   }

  //   this._data[this._selectedIndex] = {
  //     ...this._data[this._selectedIndex],
  //     nodes: this.selectedNodes,
  //     edges: this.selectedEdges
  //   };
  // }

  public addNode(type: DialogueNodeType, parentId?: string): string | null {
    let childId = null;

    switch (type) {
      case "start":
        this.addStartNode();
        break;
      case "end":
        this.addEndNode();
        break;
      case "text":
        this.addTextNode();
        break;
      case "branchContainer":
        this.addBranchContainerNode();
        break;
      case "branch": {
        childId = this.addBranchNode(parentId);
        break;
      }
      case "skillCheck": {
        childId = this.addSkillCheckNode(parentId);
        break;
      }
      default:
        throw new Error("Error creating node: Invalid node type.");
    }

    return childId;
  }

  // TODO: Replace fixed-positions with drag and drop later on.
  private addStartNode(): void {
    const newNode: StartNodeType = {
      id: crypto.randomUUID(),
      type: "start",
      position: { x: 0, y: 0 },
      data: { next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addEndNode(): void {
    const newNode: EndNodeType = {
      id: crypto.randomUUID(),
      type: "end",
      position: { x: 0, y: 0 },
      data: {}
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addTextNode(): void {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 };

    const newNode: TextNodeType = {
      id,
      type: "text",
      position,
      data: { text: "", next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addBranchContainerNode(): void {
    const id = crypto.randomUUID();
    const newNode: BranchContainerNodeType = {
      id,
      type: "branchContainer",
      position: { x: 0, y: 0 },
      data: { branches: [] },
      width: BRANCH_NODE_INITIAL_WIDTH,
      height: BRANCH_NODE_INITIAL_HEIGHT
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addBranchNode(parentId: string): string {
    const id = crypto.randomUUID();
    const newNode: BranchNodeType = {
      id,
      parentId,
      extent: "parent",
      type: "branch",
      position: { x: 0, y: 0 },
      data: { text: "", name: "", next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
    return id;
  }

  private addSkillCheckNode(parentId: string): string {
    const id = crypto.randomUUID();
    const skill = db.skillCategories?.[0]?.skills?.[0] ?? null;

    const newNode: SkillCheckNodeType = {
      id,
      parentId,
      extent: "parent",
      type: "skillCheck",
      position: { x: 0, y: 0 },
      data: { text: "", skill, difficulty: 0, next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
    return id;
  }

  // public serialize(): SerializedDialogue[] {
  //   return this.data.map((dialogue) => ({
  //     id: dialogue.id,
  //     name: dialogue.name,
  //     nodes: dialogue.nodes.map((node) => ({
  //       id: node.id,
  //       parentId: node?.parentId,
  //       extent: node?.extent,
  //       dialogueId: dialogue.id,
  //       type: node.type,
  //       positionX: node.position.x,
  //       positionY: node.position.y,
  //       width: node?.width,
  //       height: node?.height,
  //       data: JSON.stringify(node.data)
  //     })),
  //     edges: dialogue.edges.map((edge) => ({
  //       id: edge.id,
  //       dialogueId: dialogue.id,
  //       type: edge.type ?? null,
  //       source: edge.source,
  //       target: edge.target,
  //       sourceHandle: edge.sourceHandle ?? null,
  //       targetHandle: edge.targetHandle ?? null
  //     }))
  //   }));
  // }
}

export const graph = new Dialogues();
