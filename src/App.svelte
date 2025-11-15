<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { router } from './states/router.svelte';
  import type { Component } from 'svelte';

  let name = 'qlt';
  let greeting = 'hello qlt';

  async function greet() {
    greeting = await invoke('greet', { name });
  }

  // 页面组件映射
  let currentPageComponent = $state<Component | null>(null);

  // 监听页面变化并动态导入组件
  $effect(() => {
    switch (router.current) {
      case 'sign':
        import('./pages/Sign.svelte').then(module => currentPageComponent = module.default);
        break;
      case 'qr':
        import('./pages/QR.svelte').then(module => currentPageComponent = module.default);
        break;
      case 'settings':
        import('./pages/Settings.svelte').then(module => currentPageComponent = module.default);
        break;
      case 'import':
      default:
        import('./pages/Sign.svelte').then(module => currentPageComponent = module.default);
    }
  });

  // 定义过渡类
  let transitionClass = $derived({
    'slide-left': 'slide-left',
    'slide-right': 'slide-right',
    'fade': 'fade',
    'none': ''
  }[router.transition]);
</script>

<main>
  <div class={`page-container ${transitionClass}`}>
    {#if currentPageComponent}
      {@const Component = currentPageComponent}
      <Component />
    {/if}
  </div>
  
  <!-- 开发测试用 -->
  <div class="dev-tools" style="position: fixed; bottom: 10px; right: 10px; background: #fff; padding: 10px; border: 1px solid #ccc;">
    <h3>Dev Tools</h3>
    <button onclick={() => router.push('qr', {}, 'slide-left')}>Go to QR</button>
    <button onclick={() => router.push('settings', {}, 'slide-left')}>Go to Settings</button>
    <button onclick={() => router.pop('slide-right')}>Back</button>
    <p>Current: {router.current}</p>
    <p>Stack: {JSON.stringify(router.stack)}</p>
  </div>
</main>

<style>
  main {
    font-family: system-ui, sans-serif;
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .page-container {
    position: absolute;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    transition: transform 0.3s ease, opacity 0.3s ease;
  }

  .slide-left {
    transform: translateX(0);
  }

  .slide-left.slide-left-enter {
    transform: translateX(100%);
  }

  .slide-left.slide-left-leave {
    transform: translateX(-100%);
  }

  .slide-right {
    transform: translateX(0);
  }

  .slide-right.slide-right-enter {
    transform: translateX(-100%);
  }

  .slide-right.slide-right-leave {
    transform: translateX(100%);
  }

  .fade {
    opacity: 1;
  }

  .fade.fade-enter, .fade.fade-leave {
    opacity: 0;
  }
</style>