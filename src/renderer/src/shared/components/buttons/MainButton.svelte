<script lang="ts">
  export let text: string;
  export let onClick: () => void;
</script>

<button class="primary-btn" on:click|stopPropagation={onClick}>{text}</button>
