import { dialogues } from "../stores/dialogueStore.svelte";
import type { Button } from "../../../shared/types";

function editDialogue() {
  dialogues.editing = true;
}

export const editButtons: Button[] = [
  {
    text: "Add Dialogue",
    onClick: () => dialogues.add()
  },
  {
    text: "Rename Dialogue",
    onClick: editDialogue
  }
];

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
