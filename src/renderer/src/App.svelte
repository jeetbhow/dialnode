<script lang="ts">
  import { SvelteFlow, Background, Panel, MiniMap, Controls } from '@xyflow/svelte';

  import type { Button, DialogueNodeType, Portrait } from './types';

  import ButtonsContainer from './components/buttons/ButtonsContainer.svelte';
  import DialogueNode from './components/dialogueNode/DialogueNode.svelte';
  import PortraitModal from './components/portrait/PortraitModal.svelte';

  import '@xyflow/svelte/dist/style.css';

  let projectDir: string = $state('');
  let portraits: Portrait[] = $state([]);

  let nodes: DialogueNodeType[] = $state.raw([]);
  let showPortraitModal = $state(false);

  const nodeTypes = {
    dialogueNode: DialogueNode
  };

  async function selectDirectory(): Promise<void> {
    projectDir = await window.api.selectDirectory();
  }

  const closePortraitModal = () => {
    showPortraitModal = false;
  };

  const clearNodes = () => {
    nodes = [];
  };

  const addNode = () => {
    const id = crypto.randomUUID();
    const position = { x: 0, y: 0 }; // TODO: We're going to be replacing fixed-positions with drag and drop later on.

    const newDialogueNode: DialogueNodeType = {
      id,
      type: 'dialogueNode',
      position,
      data: { text: '', showOptions: false }
    };

    nodes = [...nodes, newDialogueNode];
  };

  const togglePortraitModal = () => {
    showPortraitModal = !showPortraitModal;
  };

  const controlButtons: Button[] = [
    {
      text: 'Portraits',
      onClick: togglePortraitModal
    },
    {
      text: 'Clear',
      onClick: clearNodes
    },
    {
      text: 'Project',
      onClick: selectDirectory
    }
  ];

  const nodeButtons: Button[] = [
    {
      text: '+ Node',
      onClick: addNode
    }
  ];
</script>

<div style:width="100vw" style:height="100vh">
  <SvelteFlow bind:nodes {nodeTypes} fitView>
    <MiniMap />
    <Controls />
    <Panel position="top-left">
      <ButtonsContainer flexDirection="row" buttons={nodeButtons} />
    </Panel>
    <Panel position="top-right">
      <ButtonsContainer flexDirection="row" buttons={controlButtons} />
    </Panel>
    <Background />
  </SvelteFlow>
  {#if showPortraitModal}
    <PortraitModal bind:portraits bind:projectDir onClose={closePortraitModal} />
  {/if}
</div>

<style>
  * {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  }
</style>
