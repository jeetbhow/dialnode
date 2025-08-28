import { type Edge } from "@xyflow/svelte";

import { graph } from "./graphStore.svelte";
import type { DialogueNode } from "../../../../shared/types";

export type StorageType = "folder" | "dialogue";

export interface BaseDialogueSelectNode {
  id: string;
  type: StorageType;
  name: string;
  parentId?: string;
}

export interface Dialogue extends BaseDialogueSelectNode {
  type: "dialogue";
  nodes: DialogueNode<Record<string, unknown>>[];
  edges: Edge[];
}

export interface Folder extends BaseDialogueSelectNode {
  type: "folder";
  readonly children: readonly DialogueSelectNode[];
}

export type DialogueSelectNode = Folder | Dialogue;

class RootDialogueSelectNode implements Folder {
  public editing: boolean = $state(false);

  private _selected = $state<DialogueSelectNode | null>(null);
  private _children = $state<DialogueSelectNode[]>([]);

  private _type: "folder" = "folder";
  private _id: string = "root";
  private _name: string = "root";
  private _parentId: string = null;

  get type(): "folder" {
    return this._type;
  }

  get id(): string {
    return this._id;
  }

  get name(): string {
    return this._name;
  }

  get parentId(): string {
    return this._parentId;
  }

  get selected(): DialogueSelectNode | null {
    return this._selected;
  }

  get children(): readonly DialogueSelectNode[] {
    return this._children;
  }

  public add(node: DialogueSelectNode) {
    this._children.push(node);
  }

  public remove(node: DialogueSelectNode) {
    this._children = this.children.filter((n) => n.id === node.id);
  }

  public removeSelected() {
    this._children = this._children.filter((n) => n.id !== this._selected.id);
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

export const root = new RootDialogueSelectNode();
