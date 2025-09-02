import { type Node } from "@xyflow/svelte";
import type {
  DialogueJSON,
  TextNodeType,
  BranchContainerNodeType,
  BranchNodeType,
  SkillCheckNodeType,
  StartNodeType,
  SerializedDialogue,
  SerializedDialogueNode,
  SerializedFolder
} from "../../../../shared/types";

import { Folder, type Dialogue } from "../stores/dialoguesStore.svelte";

export const BRANCH_NODE_INITIAL_WIDTH = 240;
export const BRANCH_NODE_INITIAL_HEIGHT = 240;
export const MARKER_END_WIDTH = 28;
export const MARKER_END_HEIGHT = 28;

export function serialize(root: Folder): SerializedDialogueNode[] {
  const serializedNodes: SerializedDialogueNode[] = [];
  const stack: (Folder | Dialogue)[] = [root];

  while (stack.length > 0) {
    const node = stack.pop();
    if (!node) continue;

    let serializedNode: SerializedDialogueNode;
    if (node.type === "folder") {
      serializedNode = serializeFolder(node);
      // Add children to the stack for processing, in reverse order to maintain order.
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

export function filterNodeProps(nodes: Node[]): DialogueJSON[] {
  const result = nodes.map((node) => {
    switch (node.type) {
      case "start": {
        const s = node as StartNodeType;
        return {
          id: node.id,
          type: node.type,
          next: s.data.next
        };
      }
      case "text": {
        const tn = node as TextNodeType;
        return {
          id: tn.id,
          type: tn.type,
          text: tn.data.text,
          speaker: tn.data.speaker?.name,
          portrait: tn.data.portrait?.virtualPath,
          next: tn.data.next
        };
      }
      case "branchContainer": {
        const bc = node as BranchContainerNodeType;
        return {
          id: node.id,
          type: node.type,
          branches: bc.data.branches
        };
      }
      case "branch": {
        const b = node as BranchNodeType;
        return {
          id: node.id,
          parentId: node.parentId,
          type: node.type,
          text: b.data.text,
          next: b.data.next
        };
      }
      case "skillCheck": {
        const sc = node as SkillCheckNodeType;
        return {
          id: node.id,
          parentId: node.parentId,
          type: node.type,
          text: sc.data.text,
          skill: sc.data.skill.name,
          difficulty: sc.data.difficulty,
          next: sc.data.next
        };
      }
      case "end": {
        return {
          id: node.id,
          type: node.type
        };
      }
      default: {
        throw new Error(`Invalid node type ${node.type}`);
      }
    }
  });

  return result;
}
