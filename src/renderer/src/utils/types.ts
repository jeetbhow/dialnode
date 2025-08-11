import type { Node } from "@xyflow/svelte";
import type { Speaker, Portrait, Skill } from "../stores/dbStore.svelte";

export type Button = {
  text: string;
  onClick: () => void;
};

export type DialogueData = {
  speaker?: Speaker;
  portrait?: Portrait;
  text: string;
  showOptions: boolean;
};

export type BranchData = {
  addBranch: (type: string, parentId: string) => void;
  addSkillCheck: (type: string, parentId: string) => void;
};

export type Branch = {
  name: string;
};

export type SkillCheck = {
  skill: Skill;
  difficulty: number;
};

export type DialogueNodeType = Node<DialogueData>;
export type BranchContainerNodeType = Node<BranchData>;
export type BranchNodeType = Node<Branch>;
export type SkillCheckNodeType = Node<SkillCheck>;
