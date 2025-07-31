import type { Node } from "@xyflow/svelte";

export type Button =  {
    text: string,
    onClick: () => void;
}

export type DialogueNodeData = {
    speaker?: string,
    portrait?: string,
    text: string,
}

export type DialogueNodeType = Node<DialogueNodeData>;