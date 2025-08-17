import type { Edge, Node } from "@xyflow/svelte";
import type { Speaker, Portrait, Skill } from "../../../shared/types";

export type Button = {
  text: string;
  onClick: () => void;
};

export type DialogueSelectEntry = {
  name: string;
  nodes: Node[];
  edges: Edge[];
};

export interface ConnectedTypeData extends Record<string, unknown> {
  next: string | null;
}

export interface StartData extends ConnectedTypeData {
  next: string | null;
}

export interface DialogueData extends ConnectedTypeData {
  speaker?: Speaker;
  portrait?: Portrait;
  text: string;
  showOptions: boolean;
  next: string | null;
}

export interface BranchData extends Record<string, unknown> {
  addBranch: (type: string, parentId: string) => void;
  addSkillCheck: (type: string, parentId: string) => void;
}

export interface Branch extends ConnectedTypeData {
  name: string;
  next: string | null;
}

export interface SkillCheck extends ConnectedTypeData {
  skill: Skill;
  difficulty: number;
  next: string | null;
}

export type StartNodeType = Node<StartData>;
export type DialogueNodeType = Node<DialogueData>;
export type BranchContainerNodeType = Node<BranchData>;
export type BranchNodeType = Node<Branch>;
export type SkillCheckNodeType = Node<SkillCheck>;
