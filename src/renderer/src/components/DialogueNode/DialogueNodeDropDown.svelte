<script lang="ts">
  import { onMount } from 'svelte'

  import threeDots from '../../assets/three-dots.svg'

  export let show: boolean = false
  export let isSpeakerEnabled: boolean = false
  export let isPortraitEnabled: boolean = false

  let dropdownElement: HTMLDivElement

  function handleDocumentClick(event: MouseEvent) {
    if (dropdownElement && !dropdownElement.contains(event.target as Node)) {
      show = false
    }
  }

  onMount(() => {
    document.addEventListener('click', handleDocumentClick)

    return () => {
      document.removeEventListener('click', handleDocumentClick)
    }
  })
</script>

<div class="dropdown" bind:this={dropdownElement}>
  <button class="dropdown-button" onclick={() => (show = !show)}>
    <img src={threeDots} alt="toggle dropdown" width="24" height="24" />
  </button>
  {#if show}
    <div class="dropdown-content">
      <form
        onsubmit={(e) => {
          e.preventDefault()
        }}
      >
        <div>
          <input
            type="checkbox"
            id="speaker"
            name="speaker"
            value="enabled"
            bind:checked={isSpeakerEnabled}
          />
          <label for="speaker">Speaker</label>
        </div>
        <div>
          <input
            type="checkbox"
            id="portrait"
            name="portrait"
            value="enabled"
            bind:checked={isPortraitEnabled}
          />
          <label for="portrait">Portrait</label>
        </div>
      </form>
    </div>
  {/if}
</div>

<style>
  .dropdown {
    position: relative;
    display: inline-block;
  }

  .dropdown-button {
    background: none;
    border: none;
    cursor: pointer;
  }

  .dropdown-content {
    display: block;
    position: absolute;
    background-color: #ffffff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    min-width: 160px;
    z-index: 1;
  }

  .dropdown-content form div {
    padding: 0.3rem;
    border-bottom: 1px solid #ddd;
  }
</style>
