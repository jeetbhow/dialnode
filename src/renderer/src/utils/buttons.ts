import { dialogues } from "../stores/dialogueStore.svelte";
import type { Button } from "../../../shared/types";

export const nodeButtons: Button[] = [
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
