import { graph } from "../stores/graphStore.svelte";

export const nodeButtons = [
  {
    text: "+ Start",
    onClick: graph.addStartNode
  },
  {
    text: "+ Node",
    onClick: graph.addTextNode
  },
  {
    text: "+ Branch",
    onClick: graph.addBranchContainerNode
  },
  {
    text: "+ End",
    onClick: graph.addEndNode
  }
];
