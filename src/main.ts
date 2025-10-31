import App from './App.svelte';
import { mount } from 'svelte';
import { initRouteGuards } from './lib/routeGuards';

// 初始化路由守卫
initRouteGuards();

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;