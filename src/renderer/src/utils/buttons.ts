import { dialogues } from "../stores/dialogueStore.svelte";
import type { Button } from "./types";

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
    onClick: dialogues.addStart
  },
  {
    text: "+ Node",
    onClick: dialogues.addDialogueNode
  },
  {
    text: "+ Branch",
    onClick: dialogues.addBranchContainer
  },
  {
    text: "+ End",
    onClick: dialogues.addEnd
  }
];
