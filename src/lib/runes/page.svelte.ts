// src/lib/page.svelte.ts
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
 * 页面状态机（Runes）
 */
export const page = {
  // 当前页面
  current: $state<Page>('sign'),

  // 导航堆栈（支持前进后退）
  stack: $state<Page[]>(['sign']),

  // 临时传参（如 signedTx）
  data: $state<Record<string, any>>({}),

  // ==================== 导航方法 ====================

  /** 跳转到新页面（替换当前） */
  go(to: Page, params?: Record<string, any>) {
    this.current = to;
    this.stack = [to]; // 重置堆栈
    if (params) this.data = params;
  },

  /** 前进（入栈） */
  push(to: Page, params?: Record<string, any>) {
    this.stack = [...this.stack, to];
    this.current = to;
    if (params) this.data = params;
  },

  /** 后退 */
  pop() {
    if (this.stack.length <= 1) return;
    this.stack = this.stack.slice(0, -1);
    this.current = this.stack.at(-1)!;
  },

  /** 后退到根 */
  popToRoot() {
    this.stack = [this.stack[0]];
    this.current = this.stack[0];
  },

  /** 替换当前页面 */
  replace(to: Page, params?: Record<string, any>) {
    this.stack = [...this.stack.slice(0, -1), to];
    this.current = to;
    if (params) this.data = params;
  },

  // ==================== 工具方法 ====================

  /** 是否可以后退 */
  get canGoBack() {
    return this.stack.length > 1;
  },

  /** 获取当前页面数据 */
  getData<T = any>(key: string): T | undefined {
    return this.data[key];
  },

  /** 清空临时数据 */
  clearData() {
    this.data = {};
  },

  // ==================== 高级：Rust 联动 ====================

  /** 异步跳转（配合 Rust 命令） */
  async goWithRust(to: Page, command: string, payload?: any) {
    try {
      const result = await invoke(command, payload);
      this.go(to, { result });
    } catch (error) {
      console.error('Rust command failed:', error);
    }
  },
};