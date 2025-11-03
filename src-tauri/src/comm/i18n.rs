use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// ---------- 编译期把 JSON 塞进二进制 ----------
type LocaleMap = HashMap<String, serde_json::Value>;

fn load_locale(lang: &str) -> LocaleMap {
    let raw = match lang {
        "zh" => include_str!("../../src/lib/locales/zh.json"),
        "en" => include_str!("../../src/lib/locales/en.json"),
        // 其它语言同理
        _ => include_str!("../../src/lib/locales/en.json"), // fallback
    };
    serde_json::from_str(raw).expect("invalid locale json")
}

// ---------- Tauri 状态 ----------
#[derive(Default)]
pub struct I18nState {
    current: String,
    map: LocaleMap,
}

impl I18nState {
    pub fn new(lang: &str) -> Self {
        Self {
            current: lang.to_owned(),
            map: load_locale(lang),
        }
    }
}

// ---------- 命令 ----------
#[tauri::command]
pub fn set_lang(lang: String, state: State<'_, Mutex<I18nState>>) -> Result<(), String> {
    let mut inner = state.lock().unwrap();
    *inner = I18nState::new(&lang);
    Ok(())
}

#[tauri::command]
pub fn t(
    key: String,
    params: Option<HashMap<String, String>>,
    state: State<'_, Mutex<I18nState>>,
) -> String {
    let inner = state.lock().unwrap();
    // 直接从 HashMap 中获取值
    let value = inner.map.get(&key);
    let mut text = match value {
        Some(v) => v.as_str().unwrap_or(&key).to_owned(),
        None => key.clone(),
    };

    // 替换占位符 {key}
    if let Some(p) = params {
        for (k, v) in p {
            text = text.replace(&format!("{{{}}}", k), &v);
        }
    }
    text
}
