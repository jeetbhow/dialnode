import { Node, Edge } from "@xyflow/svelte";
import { FileFilter } from "electron";

export type Repository = {
  id: string;
  name: string;
  location: string;
  godotProjectLocation: string | null;
  createdOn: string;
  appVersion: string;
  formatVersion: string;
};

export type Portrait = {
  id: string;
  kind: "portrait";
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
  kind: "speaker";
  name: string;
};

export type SkillType = "body" | "mind" | "psyche" | "sense";

export type SkillCategory = {
  id: string;
  kind: "skillCategory";
  name: string;
  skills: Skill[];
};

export type Skill = {
  id: string;
  kind: "skill";
  categoryId: string;
  name: string;
};

export type DbEntityKind = "portrait" | "speaker" | "skillCategory" | "skill";
export type DbEntity = Speaker | Portrait | SkillCategory | Skill;

export type Button = {
  text: string;
  onClick: () => void;
};

export type Dialogue = {
  id: string;
  parentId?: string;
  name: string;
  nodes: GraphNode<Record<string, unknown>>[];
  edges: Edge[];
};

export type DialogueFolder = {
  id: string;
  parentId: string;
  name: string;
  dialogues: Dialogue[];
};

export type Object = Record<string, unknown>;
export type DialogueNodeType =
  | "start"
  | "end"
  | "text"
  | "branchContainer"
  | "branch"
  | "skillCheck";

export type SerializedDialogueNode = SerializedDialogue | SerializedFolder;

export type SerializedDialogue = {
  id: string;
  parentId?: string;
  name: string;
  nodes: SerializedGraphNode[];
  edges: SerializedGraphEdge[];
};

export type SerializedFolder = {
  id: string;
  parentId?: string;
  name: string;
};

export type SerializedGraphNode = {
  id: string;
  parentId?: string;
  dialogueId: string;
  extent?: "parent";
  type: DialogueNodeType;
  positionX: number;
  positionY: number;
  width?: number;
  height?: number;
  data: string;
};

export type SerializedGraphEdge = {
  id: string;
  dialogueId: string;
  type: string;
  source: string;
  target: string;
  sourceHandle: string;
  targetHandle: string;
};

export type DialogueJSON = {
  id: string;
  type: DialogueNodeType;
};

export interface GraphNode<T extends Object> extends Node<T> {
  id: string;
  parentId?: string;
  extent?: "parent";
  type: DialogueNodeType;
  position: { x: number; y: number };
  data: T;
}

export interface Connectable extends Record<string, unknown> {
  next: string | null;
}

export interface TextNodeData extends Connectable {
  speaker?: Speaker;
  portrait?: Portrait;
  text: string;
  next: string | null;
}

export interface BranchContainerNodeData extends Record<string, unknown> {
  branches: string[];
}

export interface BranchNodeData extends Connectable {
  text: string;
  next: string | null;
}

export interface SkillCheckNodeData extends Connectable {
  text: string;
  skill: Skill;
  difficulty: number;
  next: string | null;
}

export interface EndNodeData extends Record<string, unknown> {}

export type ConnectableNodeType = GraphNode<Connectable>;
export type StartNodeType = ConnectableNodeType;
export type EndNodeType = GraphNode<EndNodeData>;
export type TextNodeType = GraphNode<TextNodeData>;
export type BranchContainerNodeType = GraphNode<BranchContainerNodeData>;
export type BranchNodeType = GraphNode<BranchNodeData>;
export type SkillCheckNodeType = GraphNode<SkillCheckNodeData>;

export type ElectronSelectDirectoryOptions = {
  title?: string;
  buttonLabel?: string;
  defaultPath?: string;
  properties?: Array<"createDirectory" | "promptToCreate">;
  filters?: FileFilter[];
};

export type ExtraSelectDirectoryOptions = {
  godot?: boolean;
};
