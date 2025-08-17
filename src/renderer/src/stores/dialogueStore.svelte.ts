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

  private _selected: number = $state(0);

  get data() {
    return this._data;
  }

  get selected() {
    return this._selected;
  }

  add(name: string) {
    this._data.push({ name, nodes: [], edges: [] });
  }

  set(index: number) {
    this._selected = index;
  }

  save(nodes: Node[], edges: Edge[]) {
    this._data[this._selected] = { ...this._data[this._selected], nodes, edges };
  }

  get(index: number) {
    return this._data[index];
  }
}

export const dialogues = new Dialogues();
