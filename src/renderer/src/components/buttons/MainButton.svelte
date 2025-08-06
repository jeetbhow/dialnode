<script lang="ts">
  import "../../styles.css";

  export let text: string;
  export let onClick: () => void;
</script>

<button on:click|stopPropagation={onClick}>{text}</button>

<style>
  button {
    color: var(--button-text-color);
    background: var(--primary-color);
    border-radius: 0.2rem;
    padding: 0.6rem 1.5rem;
  }
</style>
