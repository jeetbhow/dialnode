<script lang="ts">
  import cross from '../../assets/cross.svg';

  export let onClose: () => void;
</script>

<div class="portrait-modal">
  <header>
    <h2>Portraits</h2>
    <button onclick={onClose}>
      <img src={cross} alt="Cross" width="24" height="24" />
    </button>
  </header>
  <div>
    <!-- Portrait selection UI goes here -->
  </div>
</div>

<style>
  header {
    display: flex;
    justify-content: space-between;
    flex-flow: row nowrap;
  }

  header button {
    background: none;
    border: none;
  }

  .portrait-modal {
    position: fixed;
    top: 50%;
    left: 50%;
    width: 50%;
    height: 50%;
    transform: translate(-50%, -50%);
    background-color: white;
    padding: 0 20px;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(124, 119, 119, 0.1);
    z-index: 1000;
  }

  .portrait-modal h2 {
    font-family: 'Calibri';
  }
</style>
