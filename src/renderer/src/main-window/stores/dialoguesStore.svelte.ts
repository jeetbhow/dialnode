import { type Edge } from "@xyflow/svelte";

import { graph } from "./graphStore.svelte";
import type {
  DialogueNode,
  SerializedDialogueNode,
  SerializedDialogue,
  SerializedFolder
} from "../../../../shared/types";

export type StorageType = "folder" | "dialogue";

export interface BaseDialogueSelectNode {
  id: string;
  type: StorageType;
  name: string;
  parent: Folder | null;
}

export type DialogueSelectNode = Folder | Dialogue;

export class Dialogue implements BaseDialogueSelectNode {
  public type: "dialogue" = "dialogue";
  public nodes: DialogueNode<Record<string, unknown>>[] = [];
  public edges: Edge[] = [];

  public name = $state("");
  public parent = $state<Folder | null>(null);

  constructor(
    public id: string = crypto.randomUUID(),
    name: string = "Untitled",
    parent: Folder | null = root
  ) {
    this.name = name;
    this.parent = parent;
  }

  public move(folder: Folder) {
    if (this.parent) {
      this.parent.remove(this);
    }

    folder.add(this);
    this.parent = folder;
  }
}

export class Folder implements BaseDialogueSelectNode {
  public editing: boolean = $state(false);
  public dragged = $state<DialogueSelectNode | null>(null);
  public children = $state<DialogueSelectNode[]>([]);

  private _selected = $state<DialogueSelectNode | null>(null);

  public name = $state("root");
  public parent = $state<Folder | null>(null);

  public type: "folder" = "folder";
  public id: string;

  constructor(
    id: string = crypto.randomUUID(),
    name: string = "Untitled",
    parent: Folder | null = root
  ) {
    this.id = id;
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

  public move(folder: Folder) {
    if (this.parent) {
      this.parent.remove(this);
    }

    folder.add(this);
    this.parent = folder;
  }

  public select(node: DialogueSelectNode) {
    this.save();
    this._selected = node;
    if (node.type === "dialogue") {
      graph.display(node);
    }
  }

  public save() {
    if (this._selected !== null && this._selected.type === "dialogue") {
      this._selected.nodes = graph.nodes;
      this._selected.edges = graph.edges;
    }
  }

  public serialize(): SerializedDialogueNode[] {
    const json = [];
    const stack: Folder[] = [this];

    while (stack.length > 0) {
      // Get the top elem (always a folder)
      const top = stack.pop();

      // iterate through the children.
      for (const child of top.children) {
        // if it's a folder, then serialize it and then add it to the stack. Also
        // add it to the json.
        let ser: SerializedDialogueNode;
        if (child.type === "folder") {
          ser = this.serializeFolder(child);
          stack.push(child);
        }
        // if it's regular dialogue, then serialize it and then add it to the json.
        else {
          ser = this.serializeDialogue(child);
        }
        json.push(ser);
      }
    }

    return json;
  }

  private serializeDialogue(dialogue: Dialogue): SerializedDialogue {
    return {
      id: dialogue.id,
      parentId: dialogue.parent?.id,
      name: dialogue.name,
      nodes: dialogue.nodes.map((node) => ({
        id: node.id,
        parentId: node.parentId,
        dialogueId: dialogue.id,
        extent: node.extent,
        type: node.type,
        positionX: node.position.x,
        positionY: node.position.y,
        width: node.width,
        height: node.height,
        data: JSON.stringify(node.data)
      })),
      edges: dialogue.edges.map((edge) => ({
        id: edge.id,
        dialogueId: dialogue.id,
        type: edge.type,
        source: edge.source,
        target: edge.target,
        sourceHandle: edge.sourceHandle,
        targetHandle: edge.targetHandle
      }))
    };
  }

  private serializeFolder(folder: Folder): SerializedFolder {
    return {
      id: folder.id,
      parentId: folder.parent.id,
      name: folder.name
    };
  }
}

export const root = new Folder("root", "root", null);
