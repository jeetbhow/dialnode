import type { Node, Edge } from "@xyflow/svelte";
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
  children: DialogueSelectNode[];
}

export type DialogueSelectNode = Folder | Dialogue;

class RootFolder implements Folder {
  type: "folder" = "folder";
  id: string = "root";
  name: string = "root";
  parentId: string = null;

  selectedNodes = $state.raw<Node[]>([]);
  selectedEdges = $state.raw<Edge[]>([]);
  children = $state<DialogueSelectNode[]>([]);

  public add(node: DialogueSelectNode) {
    this.children.push(node);
  }

  public remove(node: DialogueSelectNode) {
    this.children = this.children.filter((n) => n.id === node.id);
  }
}

const rootFolder = new RootFolder();

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
  rootFolder.add(data);
}

export function useRootFolder() {
  return rootFolder;
}
