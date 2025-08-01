<script lang="ts">
  export let text: string;
  export let onClick: () => void;
</script>

<button on:click|stopPropagation={onClick}>{text}</button>

<style>
  button {
    background: none;
    border: solid 1px black;
    border-radius: 0.2rem;
    padding: 0.5rem;
    cursor: pointer;
    transition:
      background 0.2s ease-in-out,
      color 0.2s ease-in-out;
  }

  button:hover {
    background: black;
    color: white;
  }
</style>
