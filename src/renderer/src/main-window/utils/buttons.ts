import { graph } from "../stores/graphStore.svelte";

export const nodeButtons = [
  {
    text: "+ Start",
    onClick: () => graph.addNode("start")
  },
  {
    text: "+ Node",
    onClick: () => graph.addNode("text")
  },
  {
    text: "+ Branch",
    onClick: () => graph.addNode("branchContainer")
  },
  {
    text: "+ End",
    onClick: () => graph.addNode("end")
  }
];
