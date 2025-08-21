import { type Node } from "@xyflow/svelte";
import { type DialogueNodeType } from "../../../../shared/types";

export const BRANCH_NODE_INITIAL_WIDTH = 240;
export const BRANCH_NODE_INITIAL_HEIGHT = 240;
export const MARKER_END_WIDTH = 28;
export const MARKER_END_HEIGHT = 28;

type DialogueJSON = {
  id: string,
  type: DialogueNodeType,
};

export function filterNodeProps(nodes: Node[]): DialogueJSON[] {
  const result = nodes.map(node => {
    switch (node.type) {
      case "start":
        return {
          id: node.id,
          type: node.type,
          ...node.data
        };
      case "text":
        return {
          id: node.id,
          type: node.type,
          ...node.data
        };
      case "branchContainer":
        return {
          id: node.id,
          type: node.type,
          ...node.data,
        };
      case "branch":
        return {
          id: node.id,
          parentd: node.parentId,
          type: node.type,
          ...node.data,
        };
      case "skillCheck":
        return {
          id: node.id,
          parentId: node.parentId,
          type: node.type,
          ...node.data,
        };
      case "end":
        return {
          id: node.id,
          type: node.type,
        };
      default:
        throw new Error(`Invalid node type ${node.type}`);
    }
  });

  return result;
}