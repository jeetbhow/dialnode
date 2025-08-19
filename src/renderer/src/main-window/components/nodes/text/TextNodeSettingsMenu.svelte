<script lang="ts">
  import { onMount } from "svelte";

  import ThreeDots from "../../../../shared/components/icons/ThreeDots.svelte";

  const ICON_SIZE = 24;

  type Props = {
    isSpeakerEnabled: boolean;
    isPortraitEnabled: boolean;
  };

  let { isSpeakerEnabled = $bindable(), isPortraitEnabled = $bindable() }: Props = $props();
  let show: boolean = $state(false);
  let dropdownElement: HTMLDivElement;

  function handleDocumentClick(event: MouseEvent): void {
    if (dropdownElement && !dropdownElement.contains(event.target as Node)) {
      show = false;
    }
  }

  onMount(() => {
    document.addEventListener("click", handleDocumentClick);

    return () => {
      document.removeEventListener("click", handleDocumentClick);
    };
  });
</script>

<div class="dropdown" bind:this={dropdownElement}>
  <button class="dropdown-button" onclick={() => (show = !show)}>
    <ThreeDots width={ICON_SIZE} height={ICON_SIZE} />
  </button>
  {#if show}
    <div class="dropdown-content">
      <form
        onsubmit={(e) => {
          e.preventDefault();
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

  .dropdown-content {
    display: block;
    position: absolute;
    background-color: var(--node-bg-color);
    box-shadow: 0 2px 8px var(--node-bg-color);
    min-width: 160px;
    z-index: 1;
  }

  .dropdown-content form div {
    padding: 0.3rem;
    border-bottom: 1px solid var(--tab-bottom-border-color);
  }
</style>
