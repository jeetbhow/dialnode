import type { Edge } from "@xyflow/svelte";

import { useDb } from "./dbStore.svelte";
import { BRANCH_NODE_INITIAL_WIDTH, BRANCH_NODE_INITIAL_HEIGHT } from "../utils/utils";

import type {
  GraphNode,
  StartNodeType,
  TextNodeType,
  BranchNodeType,
  SkillCheckNodeType,
  EndNodeType,
  BranchContainerNodeType,
  DialogueNodeType
} from "../../../../shared/types";

import type { DialogueSelectNode } from "./dialoguesStore.svelte";

const db = useDb();

class Dialogues {
  public selected = $state<DialogueSelectNode | null>(null);
  public nodes = $state.raw<GraphNode<Record<string, unknown>>[] | null>([]);
  public edges = $state.raw<Edge[] | null>([]);

  public select(node: DialogueSelectNode): void {
    if (node.type === "dialogue") {
      this.save();
      this.nodes = node.nodes || [];
      this.edges = node.edges || [];
    }

    this.selected = node;
  }

  public clear(): void {
    this.selected = null;
    this.nodes = [];
    this.edges = [];
  }

  public save(): void {
    if (this.selected === null || this.selected.type !== "dialogue") {
      return;
    }

    this.selected.nodes = this.nodes || [];
    this.selected.edges = this.edges || [];
  }

  public removeNode(node: GraphNode<Record<string, unknown>>): void {
    this.nodes = this.nodes.filter((n) => n.id !== node.id);
    this.save();
  }

  public addNode(
    type: DialogueNodeType,
    position: { x: number; y: number },
    parentId?: string
  ): string | null {
    let childId = null;

    switch (type) {
      case "start":
        this.addStartNode(position);
        break;
      case "end":
        this.addEndNode(position);
        break;
      case "text":
        this.addTextNode(position);
        break;
      case "branchContainer":
        this.addBranchContainerNode(position);
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

    this.save();
    return childId;
  }

  // TODO: Replace fixed-positions with drag and drop later on.
  private addStartNode(position: { x: number; y: number }): void {
    const newNode: StartNodeType = {
      id: crypto.randomUUID(),
      type: "start",
      position,
      data: { next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addEndNode(position: { x: number; y: number }): void {
    const newNode: EndNodeType = {
      id: crypto.randomUUID(),
      type: "end",
      position,
      data: {}
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addTextNode(position: { x: number; y: number }): void {
    const id = crypto.randomUUID();

    const newNode: TextNodeType = {
      id,
      type: "text",
      position: position,
      data: { text: "", next: null }
    };

    graph.nodes = [...graph.nodes, newNode];
  }

  private addBranchContainerNode(position: { x: number; y: number }): void {
    const id = crypto.randomUUID();
    const newNode: BranchContainerNodeType = {
      id,
      type: "branchContainer",
      position: position,
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
}

export const graph = new Dialogues();
