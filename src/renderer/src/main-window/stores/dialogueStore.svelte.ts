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
  SerializedDialogue,
  EndNodeType,
  BranchContainerNodeType
} from "../../../../shared/types";

const db = useDb();

class Dialogues {
  public selectedNodes = $state.raw<DialogueNode<any>[]>([]);
  public selectedEdges = $state.raw<Edge[]>([]);

  private _data = $state<Dialogue[]>([]);
  private _selectedIndex: number | null = $state(null);
  private _editing: boolean = $state(false);

  get data(): Dialogue[] {
    return this._data;
  }

  get selectedIndex(): number | null {
    return this._selectedIndex;
  }

  get editing(): boolean {
    return this._editing;
  }

  set editing(value: boolean) {
    this._editing = value;
  }

  set selectedIndex(index: number) {
    this._selectedIndex = index;
    this._editing = false;
  }

  public async loadFromDb(): Promise<void> {
    const serializedDialogue = await window.api.getAllDialogues();
    const dialogues: Dialogue[] = serializedDialogue.map((sd) => ({
      ...sd,
      nodes: sd.nodes.map((node) => ({
        id: node.id,
        parentId: node.parentId ?? undefined,
        extent: node.extent ?? undefined,
        type: node.type,
        width: node.width ?? undefined,
        height: node.height ?? undefined,
        position: { x: node.positionX ?? 0, y: node.positionY ?? 0 },
        data: JSON.parse(node.data)
      }))
    }));
    this._data = dialogues;
  }

  public async saveToDb(): Promise<void> {
    this.save();
    await window.api.saveDialogues(this.serialize());
  }

  public add(): void {
    this._data.push({ id: crypto.randomUUID(), name: "Untitled", nodes: [], edges: [] });
  }

  public removeSelected(): void {
    if (this.selectedIndex === null) {
      return;
    }

    this._data = this.data.filter((_, i) => i !== this._selectedIndex);
    this.selectedIndex = this.selectedIndex !== 0 ? this.selectedIndex - 1 : null;

    if (this._selectedIndex !== null) {
      const { nodes, edges } = dialogues.get(this.selectedIndex);
      this.selectedNodes = nodes;
      this.selectedEdges = edges;
    }
  }

  public save(): void {
    if (this._selectedIndex === null) {
      return;
    }

    this._data[this._selectedIndex] = {
      ...this._data[this._selectedIndex],
      nodes: this.selectedNodes,
      edges: this.selectedEdges
    };
  }

  public get(index: number): Dialogue {
    return this._data[index];
  }

  public renameSelected(name: string): void {
    this._data[this._selectedIndex].name = name;
  }

  public selectDialogue(index: number): void {
    dialogues.save();
    dialogues.selectedIndex = index;
    const { nodes, edges } = dialogues.get(index);
    this.selectedNodes = nodes;
    this.selectedEdges = edges;
  }

  // TODO: Replace fixed-positions with drag and drop later on.
  public addStartNode(): void {
    const newNode: StartNodeType = {
      id: crypto.randomUUID(),
      type: "start",
      position: { x: 0, y: 0 },
      data: { next: null }
    };

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
  }

  public addEndNode(): void {
    const newNode: EndNodeType = {
      id: crypto.randomUUID(),
      type: "end",
      position: { x: 0, y: 0 },
      data: {}
    };

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
  }

  public addTextNode(): void {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 };

    const newNode: TextNodeType = {
      id,
      type: "text",
      position,
      data: { text: "", next: null }
    };

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
  }

  public addBranchContainerNode(): void {
    const id = crypto.randomUUID();
    const newNode: BranchContainerNodeType = {
      id,
      type: "branchContainer",
      position: { x: 0, y: 0 },
      data: { branches: [] },
      width: BRANCH_NODE_INITIAL_WIDTH,
      height: BRANCH_NODE_INITIAL_HEIGHT
    };

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
  }

  public addBranchNode(parentId: string): string {
    const id = crypto.randomUUID();
    const newNode: BranchNodeType = {
      id,
      parentId,
      extent: "parent",
      type: "branch",
      position: { x: 0, y: 0 },
      data: { text: "", name: "", next: null }
    };

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
    return id;
  }

  public addSkillCheckNode(parentId: string): string {
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

    dialogues.selectedNodes = [...dialogues.selectedNodes, newNode];
    dialogues.save();
    return id;
  }

  public serialize(): SerializedDialogue[] {
    return this.data.map((dialogue) => ({
      id: dialogue.id,
      name: dialogue.name,
      nodes: dialogue.nodes.map((node) => ({
        id: node.id,
        parentId: node?.parentId,
        extent: node?.extent,
        dialogueId: dialogue.id,
        type: node.type,
        positionX: node.position.x,
        positionY: node.position.y,
        width: node?.width,
        height: node?.height,
        data: JSON.stringify(node.data)
      })),
      edges: dialogue.edges.map((edge) => ({
        id: edge.id,
        dialogueId: dialogue.id,
        type: edge.type ?? null,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle ?? null,
        targetHandle: edge.targetHandle ?? null
      }))
    }));
  }
}

export const dialogues = new Dialogues();
