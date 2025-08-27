<script lang="ts">
  import DialogueController from "./DialogueController.svelte";
  import DialogueSelectList from "./list/DialogueSelectList.svelte";
</script>

<div class="sidebar">
  <DialogueController />
  <DialogueSelectList />
</div>

<style>
  .sidebar {
    background-color: #fafafa;
    width: 15%;
    min-width: 250px;
    height: 100%;
  }
</style>
