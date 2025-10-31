<script lang="ts">
  import { page } from './runes/states/router.svelte';

  // 定义过渡类
  $: transitionClass = {
    'slide-left': 'slide-left',
    'slide-right': 'slide-right',
    'fade': 'fade',
    'none': ''
  }[page.transition];

  // 定义页面组件映射
  const pages = {
    sign: () => import('../pages/Sign.svelte'),
    qr: () => import('../pages/QR.svelte'),
    settings: () => import('../pages/Settings.svelte'),
    import: () => import('../pages/Import.svelte'),
    export: () => import('../pages/Export.svelte'),
    about: () => import('../pages/About.svelte')
  };

  // 路由守卫示例
  const authGuard = (to: Page, from: Page) => {
    // 示例：检查用户是否已认证
    const isAuthenticated = localStorage.getItem('authenticated') === 'true';
    if (to === 'settings' && !isAuthenticated) {
      // 重定向到登录页
      page.go('sign');
      return false;
    }
    return true;
  };

  // 添加路由守卫
  page.addGuard(authGuard);
</script>

<div class="router-container">
  <div class={`page-container ${transitionClass}`}>

  </div>
</div>

<style>
  .router-container {
    position: relative;
    width: 100%;
    height: 100%;
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

  .slide-left-enter {
    transform: translateX(100%);
  }

  .slide-left-leave {
    transform: translateX(-100%);
  }

  .slide-right-enter {
    transform: translateX(-100%);
  }

  .slide-right-leave {
    transform: translateX(100%);
  }

  .fade-enter, .fade-leave {
    opacity: 0;
  }
</style>