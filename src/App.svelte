<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let name = 'qlt';
  let greeting = 'hello qlt';

  async function greet() {
    greeting = await invoke('greet', { name });
  }
</script>

<main >
  <h1>Hello from Svelte + Tauri!</h1>
  <input bind:value={name} placeholder="Enter your name" />
  <button on:click={greet}>Greet</button>
  {#if greeting}
    <p>{greeting}</p>
  {/if}
</main>

<style>
  main {
    font-family: system-ui, sans-serif;
  }
</style>
