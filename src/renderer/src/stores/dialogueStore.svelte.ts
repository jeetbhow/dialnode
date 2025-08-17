import type { Node, Edge } from "@xyflow/svelte";
import type { DialogueSelectEntry } from "../utils/types";

class Dialogues {
  private _data = $state<DialogueSelectEntry[]>([
    {
      name: "Dialogue 1",
      nodes: [],
      edges: []
    },
    {
      name: "Dialogue 2",
      nodes: [],
      edges: []
    }
  ]);

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
    // This looks weird but all it's doing is clamping the selected index so that
    // it doesn't go out of bounds of the array.
    this._index = Math.max(0, Math.min(this._index, this.data.length - 1));
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
}

export const dialogues = new Dialogues();
