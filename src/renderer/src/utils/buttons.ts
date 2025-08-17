import { dialogues } from "../stores/dialogueStore.svelte";
import type { Button } from "./types";

function editDialogue() {
  dialogues.editing = true;
}

const editButtons: Button[] = [
  {
    text: "Add Dialogue",
    onClick: () => dialogues.add()
  },
  {
    text: "Rename Dialogue",
    onClick: editDialogue
  }
];
