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
  path: string;
};
