import type { Node, Edge } from "@xyflow/svelte";
import type { DialogueSelectEntry } from "../utils/types";

class Dialogues {
  public nodes = $state.raw<Node[]>([]);
  public edges = $state.raw<Edge[]>([]);

  private _data = $state<DialogueSelectEntry[]>([]);

  private _index: number = $state(0);
  private _editing: boolean = $state(false);

  get data(): DialogueSelectEntry[] {
    return this._data;
  }

  get index(): number {
    return this._index;
  }

  get editing(): boolean {
    return this._editing;
  }

  set editing(value: boolean) {
    this._editing = value;
  }

  set index(index: number) {
    this._index = index;
    this._editing = false;
  }

  public add(): void {
    this._data.push({ name: "Untitled", nodes: [], edges: [] });
  }

  public removeSelected(): void {
    this._data = this.data.filter((_, i) => i !== this._index);
    this.index = Math.max(0, --this.index);
    const { nodes, edges } = dialogues.get(this.index);
    this.nodes = nodes;
    this.edges = edges;
  }

  public save(nodes: Node[], edges: Edge[]): void {
    this._data[this._index] = { ...this._data[this._index], nodes, edges };
  }

  public get(index: number): DialogueSelectEntry {
    return this._data[index];
  }

  public renameSelected(name: string) {
    this._data[this._index].name = name;
  }

  public selectDialogue(index: number) {
    dialogues.save(this.nodes, this.edges);
    dialogues.index = index;
    const { nodes, edges } = dialogues.get(index);
    this.nodes = nodes;
    this.edges = edges;
  }
}

export const dialogues = new Dialogues();
