import { dialogues } from "../stores/dialogueStore.svelte";

export const nodeButtons = [
  {
    text: "+ Start",
    onClick: dialogues.addStartNode
  },
  {
    text: "+ Node",
    onClick: dialogues.addTextNode
  },
  {
    text: "+ Branch",
    onClick: dialogues.addBranchContainerNode
  },
  {
    text: "+ End",
    onClick: dialogues.addEndNode
  }
];
