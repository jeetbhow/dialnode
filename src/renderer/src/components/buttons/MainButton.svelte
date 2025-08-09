<script lang="ts">
  import "../../styles.css";

  export let text: string;
  export let onClick: () => void;
</script>

<button class="primary-btn" on:click|stopPropagation={onClick}>{text}</button>
