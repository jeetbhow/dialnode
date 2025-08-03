<script lang="ts">
  export let text: string;
  export let onClick: () => void;
</script>

<button on:click|stopPropagation={onClick}>{text}</button>

<style>
  button {
    color: rgb(78, 96, 249);
    background: none;
    border: solid 1px rgb(78, 96, 249);
    border-radius: 0.2rem;
    padding: 0.6rem 1.5rem;
    cursor: pointer;
    transition:
      background 150ms ease-in-out,
      color 150ms ease-in-out;
  }

  button:hover {
    background: rgb(78, 96, 249);
    color: rgb(205, 198, 247);
  }
</style>
