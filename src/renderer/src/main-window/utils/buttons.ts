import type { DialogueNodeType } from "../../../../shared/types";

export type Button = {
  text: string;
  nodeType?: DialogueNodeType;
};

export const nodeButtons = [
  {
    text: "+ Start",
    nodeType: "start"
  },
  {
    text: "+ Node",
    nodeType: "text"
  },
  {
    text: "+ Branch",
    nodeType: "branchContainer"
  },
  {
    text: "+ End",
    nodeType: "end"
  }
] satisfies Button[];
