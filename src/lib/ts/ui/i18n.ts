import { invoke } from '@tauri-apps/api/core';

// 当前语言（可写进 localStorage 持久化）
export let currentLang = $state('en');

export async function setLang(lang: string) {
    await invoke('set_lang', { lang });
    currentLang = lang;
    // 可选：localStorage.setItem('lang', lang);
}

// 核心翻译函数
export async function t(key: string, params?: Record<string, string>): Promise<string> {
    return await invoke('t', { key, params });
}

// ---------- Svelte 专用 reactive store ----------
import { readable } from 'svelte/store';

export const $t = readable(
    async (key: string, params?: Record<string, string>) => await t(key, params),
    (set) => {
        // 这里只是占位，实际在组件里用 `await $t(...)`
    }
);