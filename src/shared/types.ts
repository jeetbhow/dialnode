import { Node, Edge, type CoordinateExtent } from "@xyflow/svelte";
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
  name: string;
  nodes: DialogueNode<any>[];
  edges: Edge[];
};

export type Object = Record<string, unknown>;
export type DialogueNodeType =
  | "start"
  | "end"
  | "text"
  | "branchContainer"
  | "branch"
  | "skillCheck";

export type SerializedDialogue = {
  id: string;
  name: string;
  nodes: SerializedDialogueNode[];
  edges: SerializedDialogueEdge[];
};

export type SerializedDialogueNode = {
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

export type SerializedDialogueEdge = {
  id: string;
  dialogueId: string;
  type: string;
  source: string;
  target: string;
  sourceHandle: string;
  targetHandle: string;
};

export interface DialogueNode<T extends Object> extends Node<T> {
  id: string;
  parentId?: string;
  extent?: "parent",
  type: DialogueNodeType;
  position: { x: number; y: number; };
  data: T;
}

export interface StartNodeData extends Record<string, unknown> {
  next: string;
}

export interface TextNodeData extends Record<string, unknown> {
  speaker?: Speaker;
  portrait?: Portrait;
  text: string;
  next: string | null;
}

export interface BranchContainerNodeData extends Record<string, unknown> {
  next: string[];
}

export interface BranchNodeData extends Record<string, unknown> {
  text: string;
  next: string | null;
}

export interface SkillCheckNodeData extends Record<string, unknown> {
  text: string;
  skill: Skill;
  difficulty: number;
  next: string | null;
}

export interface EndNodeData extends Record<string, unknown> { }

export type EndNodeType = DialogueNode<EndNodeData>;
export type StartNodeType = DialogueNode<StartNodeData>;
export type TextNodeType = DialogueNode<TextNodeData>;
export type BranchContainerNodeType = DialogueNode<BranchContainerNodeData>;
export type BranchNodeType = DialogueNode<BranchNodeData>;
export type SkillCheckNodeType = DialogueNode<SkillCheckNodeData>;

export type ElectronSelectDirectoryOptions = {
  title?: string;
  buttonLabel?: string;
  defaultPath?: string;
  properties?: Array<'createDirectory' | 'promptToCreate'>;
  filters?: FileFilter[];
};

export type ExtraSelectDirectoryOptions = {
  godot?: boolean;
};