import type { Node, Edge } from "@xyflow/svelte";

import { useDb } from "./dbStore.svelte";
import { BRANCH_NODE_INITIAL_WIDTH, BRANCH_NODE_INITIAL_HEIGHT } from "../utils/utils";

import type {
  DialogueSelectEntry,
  StartNodeType,
  DialogueNodeType,
  BranchNodeType,
  SkillCheckNodeType
} from "../utils/types";

const db = useDb();

class Dialogues {
  public nodes = $state.raw<Node[]>([]);
  public edges = $state.raw<Edge[]>([]);

  private _data = $state<DialogueSelectEntry[]>([]);
  private _selectedIndex: number | null = $state(null);
  private _editing: boolean = $state(false);

  get data(): DialogueSelectEntry[] {
    return this._data;
  }

  get selectedIndex(): number {
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

  public add(): void {
    this._data.push({ name: "Untitled", nodes: [], edges: [] });
  }

  public removeSelected(): void {
    if (this.selectedIndex === null) {
      return;
    }

    this._data = this.data.filter((_, i) => i !== this._selectedIndex);
    this.selectedIndex = this.selectedIndex !== 0 ? this.selectedIndex - 1 : null;

    const { nodes, edges } = dialogues.get(this.selectedIndex);
    this.nodes = nodes;
    this.edges = edges;
  }

  public save(nodes: Node[], edges: Edge[]): void {
    this._data[this._selectedIndex] = { ...this._data[this._selectedIndex], nodes, edges };
  }

  public get(index: number): DialogueSelectEntry {
    return this._data[index];
  }

  public renameSelected(name: string) {
    if (this.selectedIndex === null) {
      return;
    }

    this._data[this._selectedIndex].name = name;
  }

  public selectDialogue(index: number) {
    dialogues.save(this.nodes, this.edges);
    dialogues.selectedIndex = index;
    const { nodes, edges } = dialogues.get(index);
    this.nodes = nodes;
    this.edges = edges;
  }

  // TODO: Replace fixed-positions with drag and drop later on.
  public addStart(): void {
    const newNode: StartNodeType = {
      id: crypto.randomUUID(),
      type: "start",
      position: { x: 0, y: 0 },
      data: { next: "" }
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }

  public addEnd(): void {
    const newNode = {
      id: crypto.randomUUID(),
      type: "end",
      position: { x: 0, y: 0 },
      data: {}
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }

  public addDialogueNode(): void {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 };

    const newNode: DialogueNodeType = {
      id,
      type: "dialogue",
      position,
      data: { text: "", showOptions: false, next: null }
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }

  public addBranchContainer(): void {
    const id = crypto.randomUUID();
    const newNode: Node = {
      id,
      type: "branchContainer",
      position: { x: 0, y: 0 },
      data: {},
      width: BRANCH_NODE_INITIAL_WIDTH,
      height: BRANCH_NODE_INITIAL_HEIGHT
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }

  public addBranch(parentId: string): void {
    const newNode: BranchNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "branch",
      position: { x: 0, y: 0 },
      data: { name: "", next: "" }
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }

  public addSkillCheck(parentId: string): void {
    const newNode: SkillCheckNodeType = {
      id: crypto.randomUUID(),
      parentId,
      extent: "parent",
      type: "skillCheck",
      position: { x: 0, y: 0 },
      data: { skill: db.skillCategories[0].skills[0], difficulty: 0, next: "" }
    };

    dialogues.nodes = [...dialogues.nodes, newNode];
  }
}

export const dialogues = new Dialogues();
