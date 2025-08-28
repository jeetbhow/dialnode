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

  private _selectedDialogue = $state<Dialogue | null>(null);
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

  get selectedDialogue(): Dialogue | null {
    return this._selectedDialogue;
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
    this._children = this._children.filter((n) => n.id !== this._selectedDialogue.id);
    this._selectedDialogue = null;
  }

  public selectDialogue(dialogue: Dialogue) {
    // This fetches the nodes and edges from the flow and updates
    // the data on the currently selected dialogue before discarding it.
    if (this._selectedDialogue !== null) {
      this._selectedDialogue.nodes = graph.nodes;
      this._selectedDialogue.edges = graph.edges;
    }

    this._selectedDialogue = dialogue;
    graph.display(dialogue);
  }
}

export const root = new RootDialogueSelectNode();

const testData: DialogueSelectNode[] = [
  {
    id: "1",
    type: "folder",
    name: "folder 1",
    children: [
      {
        id: "folder 1 child folder",
        parentId: "1",
        type: "folder",
        name: "folder 1 child folder",
        children: []
      },
      {
        id: "folder 1 child dialogue",
        parentId: "1",
        type: "dialogue",
        name: "folder 1 child dialogue",
        edges: [],
        nodes: []
      }
    ]
  },
  {
    id: "2",
    type: "dialogue",
    name: "dialogue 1",
    nodes: [],
    edges: []
  }
];

for (const data of testData) {
  root.add(data);
}
