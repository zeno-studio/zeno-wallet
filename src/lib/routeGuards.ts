// src/lib/routeGuards.ts
import { router, type Page } from './runes/states/router.svelte';

// 认证守卫 - 检查用户是否已登录
export const authGuard = async (to: Page, from: Page): Promise<boolean> => {
  // 模拟检查认证状态
  const isAuthenticated = !!localStorage.getItem('authToken');
  
  // 如果要访问受保护的页面但未认证，则重定向到登录页
  if (!isAuthenticated && ['settings', 'export'].includes(to)) {
    console.log('未认证用户试图访问受保护的页面，重定向到登录页');
    router.go('sign');
    return false;
  }
  
  return true;
};

// 确认离开守卫 - 在离开某些页面前询问用户
export const confirmLeaveGuard = async (to: Page, from: Page): Promise<boolean> => {
  // 如果从表单页面离开，询问用户是否确认
  if (['import', 'export'].includes(from) && router.getData('hasUnsavedChanges')) {
    const confirmed = confirm('您有未保存的更改，确定要离开吗？');
    if (!confirmed) {
      return false;
    }
  }
  
  return true;
};

// 初始化路由守卫
export const initRouteGuards = () => {
  router.addGuard(authGuard);
  router.addGuard(confirmLeaveGuard);
};