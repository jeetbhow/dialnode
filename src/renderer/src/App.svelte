<script lang="ts">
  import type { Button, DialogueNodeType } from './types'
  import { SvelteFlow, Background, Panel, MiniMap, Controls } from '@xyflow/svelte'

  import ButtonsContainer from './components/ButtonsContainer.svelte'
  import DialogueNode from './components/DialogueNode.svelte'

  import '@xyflow/svelte/dist/style.css'

  const nodeTypes = {
    dialogueNode: DialogueNode
  }

  let nodes: DialogueNodeType[] = $state.raw([])

  const clearNodes = () => {
    nodes = []
  }

  const addNode = () => {
    const id = crypto.randomUUID()
    const position = { x: 0, y: 0 } // TODO: We're going to be replacing fixed-positions with drag and drop later on.

    const newDialogueNode: DialogueNodeType = {
      id,
      type: 'dialogueNode',
      position,
      data: { text: '' }
    }

    nodes = [...nodes, newDialogueNode]
  }

  const buttons: Button[] = [
    {
      text: '+ Node',
      onClick: addNode
    },
    {
      text: 'Clear',
      onClick: clearNodes
    }
  ]
</script>

<div style:width="100vw" style:height="100vh">
  <SvelteFlow bind:nodes {nodeTypes} fitView>
    <MiniMap />
    <Controls />
    <Panel position="top-left">
      <ButtonsContainer flexDirection="row" {buttons} />
    </Panel>
    <Background />
  </SvelteFlow>
</div>
