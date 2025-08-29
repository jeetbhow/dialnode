import { type Edge } from "@xyflow/svelte";

import { graph } from "./graphStore.svelte";
import type { DialogueNode } from "../../../../shared/types";

export type StorageType = "folder" | "dialogue";

export interface BaseDialogueSelectNode {
  id: string;
  type: StorageType;
  name: string;
  parent: Folder | null;
}

export interface Dialogue extends BaseDialogueSelectNode {
  type: "dialogue";
  nodes: DialogueNode<Record<string, unknown>>[];
  edges: Edge[];
}

export interface Folder extends BaseDialogueSelectNode {
  type: "folder";
  children: DialogueSelectNode[];
  add: (node: DialogueSelectNode) => void;
  remove: (node: DialogueSelectNode) => void;
}

export type DialogueSelectNode = Folder | Dialogue;

export class DialogueFolder implements Folder {
  public editing: boolean = $state(false);
  public dragged = $state<DialogueSelectNode | null>(null);
  public children = $state<DialogueSelectNode[]>([]);

  private _selected = $state<DialogueSelectNode | null>(null);

  public type: "folder" = "folder";
  public id: string = "root";
  public name: string = "root";
  public parent: Folder | null = null;

  constructor(id: string = "root", name: string = "root", parent: Folder | null = null) {
    this.id = id;
    this.type = "folder";
    this.name = name;
    this.parent = parent;
  }

  get selected(): DialogueSelectNode | null {
    return this._selected;
  }

  public add(node: DialogueSelectNode) {
    this.children.push(node);
  }

  public remove(node: DialogueSelectNode) {
    this.children = this.children.filter((n) => n.id !== node.id);
  }

  public removeSelected() {
    this.children = this.children.filter((n) => n.id !== this._selected.id);
    this._selected = null;
  }

  public select(node: DialogueSelectNode) {
    // This fetches the nodes and edges from the flow and updates
    // the data on the currently selected dialogue before discarding it.
    if (this._selected !== null && this._selected.type === "dialogue") {
      this._selected.nodes = graph.nodes;
      this._selected.edges = graph.edges;
    }

    this._selected = node;
    if (node.type === "dialogue") {
      graph.display(node);
    }
  }
}

export const root = new DialogueFolder();
