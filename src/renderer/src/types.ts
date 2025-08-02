import type { Node } from '@xyflow/svelte';

export type Button = {
  text: string;
  onClick: () => void;
};

export type DialogueNodeData = {
  speaker?: string;
  portrait?: string;
  text: string;
  showOptions: boolean;
};

export type DialogueNodeType = Node<DialogueNodeData>;

export type Portrait = {
  id: string;
  name: string;
  dataURL: string;
  width: number;
  height: number;
  path: string;
  filename: string;
};
