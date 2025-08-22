import { type Node } from "@xyflow/svelte";
import type {
  DialogueJSON,
  TextNodeType,
  BranchContainerNodeType,
  BranchNodeType,
  SkillCheckNodeType,
  StartNodeType
} from "../../../../shared/types";

export const BRANCH_NODE_INITIAL_WIDTH = 240;
export const BRANCH_NODE_INITIAL_HEIGHT = 240;
export const MARKER_END_WIDTH = 28;
export const MARKER_END_HEIGHT = 28;

export function filterNodeProps(nodes: Node[]): DialogueJSON[] {
  const result = nodes.map((node) => {
    switch (node.type) {
      case "start":
        const s = node as StartNodeType;
        return {
          id: node.id,
          type: node.type,
          next: s.data.next
        };
      case "text":
        const tn = node as TextNodeType;
        return {
          id: tn.id,
          type: tn.type,
          text: tn.data.text,
          speaker: tn.data.speaker?.name,
          portrait: tn.data.portrait?.virtualPath,
          next: tn.data.next
        };
      case "branchContainer":
        const bc = node as BranchContainerNodeType;
        return {
          id: node.id,
          type: node.type,
          branches: bc.data.branches
        };
      case "branch":
        const b = node as BranchNodeType;
        return {
          id: node.id,
          parentd: node.parentId,
          type: node.type,
          text: b.data.text,
          next: b.data.next
        };
      case "skillCheck":
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
      case "end":
        return {
          id: node.id,
          type: node.type
        };
      default:
        throw new Error(`Invalid node type ${node.type}`);
    }
  });

  return result;
}
