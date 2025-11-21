import { invoke } from '@tauri-apps/api/core';
import type { Locales, Locale } from './types';


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

export function getLocaleName(locale: string): string {
  switch (locale) {
    case "zh-CN": return "简体中文";
    case "zh-TW": return "繁體中文";
    case "ja-JP": return "日本語";
    case "ko-KR": return "한국어";

    case "en-SG": return "English (Singapore)";
    case "vi-VN": return "Tiếng Việt";
    case "ms-MY": return "Bahasa Malaysia";
    case "id-ID": return "Bahasa Indonesia";

    case "en-IN": return "English (India)";
    case "hi-IN": return "हिन्दी";

    case "es-AR": return "Español (Argentina)";
    case "es-VE": return "Español (Venezuela)";
    case "pt-BR": return "Português (Brasil)";

    case "en-US": return "English (US)";
    case "es-ES": return "Español (España)";
    case "fr-FR": return "Français";
    case "de-DE": return "Deutsch";
    case "ru-RU": return "Русский";

    default: return locale;
  }
}

export function getLocaleFlag(locale: Locale): string {
  switch (locale) {
    case "zh-CN": return "🇨🇳";
    case "zh-TW": return "🇹🇼";
    case "ja-JP": return "🇯🇵";
    case "ko-KR": return "🇰🇷";

    case "en-SG": return "🇸🇬";
    case "vi-VN": return "🇻🇳";
    case "ms-MY": return "🇲🇾";
    case "id-ID": return "🇮🇩";

    case "en-IN": return "🇮🇳";
    case "hi-IN": return "🇮🇳";

    case "es-AR": return "🇦🇷";
    case "es-VE": return "🇻🇪";
    case "pt-BR": return "🇧🇷";

    case "en-US": return "🇺🇸";
    case "es-ES": return "🇪🇸";
    case "fr-FR": return "🇫🇷";
    case "de-DE": return "🇩🇪";
    case "ru-RU": return "🇷🇺";

    default: return "🏳️";
  }
}

export function detectLocale(supported: Locales): string {
  const browser = navigator.language || "en-US";

  // 完全匹配
  if (supported[browser as keyof Locales]) {
    return browser;
  }

  const base = browser.split("-")[0];

  // 基于主语种匹配
  const fallbackMapping: Record<string, Locale> = {
    "zh": "zh-CN",
    "en": "en-US",
    "es": "es-ES",
    "pt": "pt-BR",
    "fr": "fr-FR",
    "de": "de-DE",
    "ja": "ja-JP",
    "ko": "ko-KR",
    "vi": "vi-VN",
    "id": "id-ID",
    "ms": "ms-MY",
    "hi": "hi-IN",
    "ru": "ru-RU",
  };

  const mapped = fallbackMapping[base];
  if (mapped && supported[mapped as keyof Locales]) {
    return mapped;
  }

  // 最终 fallback
  return "en-US";
}

export function getLocaleLogo(locale: Locale): string {
  return `./${locale}.svg`;
}
