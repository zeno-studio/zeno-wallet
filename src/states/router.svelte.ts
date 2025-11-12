// src/lib/runes/router.svelte.ts
import { invoke } from '@tauri-apps/api/core';

/**
 * 页面类型定义（可扩展）
 */
export type Page =
  | 'sign'
  | 'qr'
  | 'settings'
  | 'import'
  | 'export'
  | 'about';

/**
 * 路由切换动画类型
 */
export type TransitionType = 
  | 'slide-left'
  | 'slide-right'
  | 'fade'
  | 'none';

/**
 * 路由守卫函数类型
 */
export type RouteGuard = (to: Page, from: Page) => boolean | Promise<boolean>;

/**
 * 页面状态机（Runes）
 */
class Router {
  // 当前页面
  current: Page = $state<Page>('sign');

  // 导航堆栈（支持前进后退）
  stack: Page[] = $state<Page[]>(['sign']);

  // 临时传参（如 signedTx）
  data: Record<string, any> = $state<Record<string, any>>({});

  // 页面切换动画
  transition: TransitionType = $state<TransitionType>('none');
  
  // 上一个页面（用于动画）
  previous: Page | null = $state<Page | null>(null);

  // 路由守卫
  guards: RouteGuard[] = [];

  // ==================== 导航方法 ====================

  /** 跳转到新页面（替换当前） */
  go(to: Page, params?: Record<string, any>, transition: TransitionType = 'none') {
    this.previous = this.current;
    this.transition = transition;
    this.current = to;
    this.stack = [to]; // 重置堆栈
    if (params) this.data = params;
  };

  /** 前进（入栈） */
  push(to: Page, params?: Record<string, any>, transition: TransitionType = 'slide-left') {
    this.previous = this.current;
    this.transition = transition;
    this.stack = [...this.stack, to];
    this.current = to;
    if (params) this.data = params;
  };

  /** 后退 */
  pop(transition: TransitionType = 'slide-right') {
    if (this.stack.length <= 1) return;
    this.previous = this.current;
    this.transition = transition;
    this.stack = this.stack.slice(0, -1);
    this.current = this.stack[this.stack.length - 1];
  };

  /** 后退到根 */
  popToRoot(transition: TransitionType = 'slide-right') {
    if (this.stack.length <= 1) return;
    this.previous = this.current;
    this.transition = transition;
    this.stack = [this.stack[0]];
    this.current = this.stack[0];
  };

  /** 替换当前页面 */
  replace(to: Page, params?: Record<string, any>, transition: TransitionType = 'none') {
    this.previous = this.current;
    this.transition = transition;
    this.stack = [...this.stack.slice(0, -1), to];
    this.current = to;
    if (params) this.data = params;
  };

  // ==================== 路由守卫 ====================

  /** 添加路由守卫 */
  addGuard(guard: RouteGuard) {
    this.guards.push(guard);
  };

  /** 移除路由守卫 */
  removeGuard(guard: RouteGuard) {
    const index = this.guards.indexOf(guard);
    if (index > -1) {
      this.guards.splice(index, 1);
    }
  };

  /** 检查是否可以通过路由守卫 */
  async canActivate(to: Page): Promise<boolean> {
    const from = this.current;
    for (const guard of this.guards) {
      try {
        const result = await guard(to, from);
        if (!result) return false;
      } catch (error) {
        console.error('Route guard error:', error);
        return false;
      }
    }
    return true;
  };

  /** 带守卫的导航 */
  async guardedPush(to: Page, params?: Record<string, any>, transition: TransitionType = 'slide-left') {
    if (await this.canActivate(to)) {
      this.push(to, params, transition);
      return true;
    }
    return false;
  };

  /** 带守卫的跳转 */
  async guardedGo(to: Page, params?: Record<string, any>, transition: TransitionType = 'none') {
    if (await this.canActivate(to)) {
      this.go(to, params, transition);
      return true;
    }
    return false;
  };

  // ==================== 工具方法 ====================

  /** 是否可以后退 */
  get canGoBack() {
    return this.stack.length > 1;
  }

  /** 获取当前页面数据 */
  getData<T = any>(key: string): T | undefined {
    return this.data[key];
  };

  /** 清空临时数据 */
  clearData() {
    this.data = {};
  };

  /** 重置路由状态 */
  reset() {
    this.current = 'sign';
    this.stack = ['sign'];
    this.data = {};
    this.transition = 'none';
    this.previous = null;
  };

  // ==================== 高级：Rust 联动 ====================

  /** 异步跳转（配合 Rust 命令） */
  async goWithRust(to: Page, command: string, payload?: any, transition: TransitionType = 'none') {
    try {
      const result = await invoke(command, payload);
      this.go(to, { result }, transition);
    } catch (error) {
      console.error('Rust command failed:', error);
    }
  };
}

export const router = new Router();