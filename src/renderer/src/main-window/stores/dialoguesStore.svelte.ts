import { type Edge } from "@xyflow/svelte";

import type {
  GraphNode,
  SerializedDialogue,
  SerializedDialogueNode,
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
  public nodes: GraphNode<Record<string, unknown>>[] = [];
  public edges: Edge[] = [];

  public name = $state("");
  public parent = $state<Folder | null>(null);

  constructor(
    public id: string = crypto.randomUUID(),
    name: string = "Untitled",
    parent: Folder | null = null
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
  public children = $state<DialogueSelectNode[]>([]);
  public name = $state("root");
  public parent = $state<Folder | null>(null);

  public type: "folder" = "folder";
  public id: string;

  constructor(
    id: string = crypto.randomUUID(),
    name: string = "Untitled",
    parent: Folder | null = null
  ) {
    this.id = id;
    this.name = name;
    this.parent = parent;
  }

  public add(node: DialogueSelectNode) {
    this.children.push(node);
  }

  public remove(node: DialogueSelectNode) {
    this.children = this.children.filter((n) => n.id !== node.id);
  }

  public move(folder: Folder) {
    if (this.parent) {
      this.parent.remove(this);
    }

    folder.add(this);
    this.parent = folder;
  }
}

export async function saveToDb(): Promise<void> {
  await window.api.saveDialogues(serialize(root));
}

export async function loadDialoguesFromDb(): Promise<void> {
  const serializedNodes = await window.api.getAllDialogues();
  const nodeMap = new Map<string, DialogueSelectNode>();

  root.children = [];
  nodeMap.set(root.id, root);

  // First pass: create all nodes and add them to the map
  for (const sNode of serializedNodes) {
    if (sNode.id === "root") continue;

    let node: DialogueSelectNode;
    if (sNode.type === "dialogue") {
      const dialogue = new Dialogue(sNode.id, sNode.name);
      dialogue.nodes = sNode.nodes.map((n) => ({
        id: n.id,
        type: n.type,
        position: { x: n.positionX, y: n.positionY },
        data: JSON.parse(n.data),
        width: n.width ?? undefined,
        height: n.height ?? undefined,
        parentId: n.parentId ?? undefined,
        extent: n.extent ?? undefined
      }));
      dialogue.edges = sNode.edges;
      node = dialogue;
    } else {
      node = new Folder(sNode.id, sNode.name);
    }

    nodeMap.set(node.id, node);
  }

  // Second pass: link children to their parents
  for (const sNode of serializedNodes) {
    if (sNode.id === "root") continue;

    const childNode = nodeMap.get(sNode.id);
    if (!childNode) continue;

    const parentNode = nodeMap.get(sNode.parentId ?? "root") as Folder;
    if (parentNode) {
      parentNode.add(childNode);
      childNode.parent = parentNode;
    } else {
      // This case should ideally not happen if parentId is always valid or null
      root.add(childNode);
      childNode.parent = root;
    }
  }
}

export function serialize(root: Folder): SerializedDialogueNode[] {
  const serializedNodes: SerializedDialogueNode[] = [];
  const stack: (Folder | Dialogue)[] = [root];

  while (stack.length > 0) {
    const node = stack.pop();
    if (!node) continue;

    let serializedNode: SerializedDialogueNode;
    if (node.type === "folder") {
      serializedNode = serializeFolder(node);
      const children = [...node.children].reverse();
      for (const child of children) {
        stack.push(child);
      }
    } else {
      serializedNode = serializeDialogue(node);
    }
    serializedNodes.push(serializedNode);
  }

  return serializedNodes;
}

function serializeDialogue(dialogue: Dialogue): SerializedDialogue {
  return {
    id: dialogue.id,
    parentId: dialogue.parent?.id,
    type: "dialogue",
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

function serializeFolder(folder: Folder): SerializedFolder {
  return {
    id: folder.id,
    parentId: folder.parent?.id ?? null,
    type: "folder",
    name: folder.name
  };
}

export const root = new Folder("root", "root", null);

type RootState = {
  editing: boolean;
  draggedNode: DialogueSelectNode;
};

export let rootState = $state<RootState>({
  editing: false,
  draggedNode: null
});
