<script lang='ts'>
    export let text: string;
    export let onClick: () => void; 
</script>

<button on:click={onClick}>{text}</button>