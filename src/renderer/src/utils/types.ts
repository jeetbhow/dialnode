import type { Node } from "@xyflow/svelte";

export type DbRequestType = "Portrait" | "Speaker";

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
};

export type Branch = {
  name: string;
};

export type DialogueNodeType = Node<DialogueData>;
export type BranchContainerNodeType = Node<BranchData>;
export type BranchNodeType = Node<Branch>;

export type Portrait = {
  id: string;
  name: string;
  dataURL: string;
  width: number;
  height: number;
  path: string;
  relPath: string;
  virtualPath: string;
  filename: string;
};

export type Speaker = {
  id: string;
  name: string;
};
