use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::Mutex;
use z_wallet_core::{WalletCore, constants};

use crate::core::db::{App, DB_INSTANCE, TableKind, TableManager, UiConfig, vault_get};
use crate::error::{AppError, DbResult};
use rust_rocksdb::WriteBatch;
pub struct AppState {
    pub wallet: Mutex<WalletCore>,
    pub ui_config: Mutex<UiConfig>,
}

pub enum VaultType {
    V1,
}
impl VaultType {
    pub fn to_string(&self) -> String {
        match self {
            VaultType::V1 => constants::VERSION_TAG_1.to_string(),
        }
    }
}

impl AppState {
    pub fn init() -> Result<AppState, AppError> {
        let mut wallet = WalletCore::default();
        let mut ui_config = UiConfig::default();

        if let Ok(Some(init)) = config_get("is_initialized".to_string()) {
            if init == "true" {
                ui_config = config_batch_get()?;
                if let Some(vault) = vault_get(VaultType::V1.to_string()).unwrap() {
                    wallet = WalletCore {
                        version: Some(vault.version),
                        salt: Some(vault.salt),
                        nonce: Some(vault.nonce),
                        ciphertext: Some(vault.ciphertext),
                        derived_key: None,
                        expire_time: None,
                        cache_duration: Some(constants::DEFAULT_CACHE_DURATION),
                        entropy_bits: Some(constants::DEFAULT_ENTROPY_BITS),
                    };
                }
            }
        }

        Ok(AppState {
            wallet: Mutex::new(wallet),
            ui_config: Mutex::new(ui_config),
        })
    }

    /// 在 Tauri 应用启动时初始化 AppState
    pub fn tauri_setup(_app_handle: &tauri::AppHandle) -> Result<(), AppError> {
        let _ = &*APP_STATE;
        Ok(())
    }
}

pub static APP_STATE: Lazy<Arc<AppState>> = Lazy::new(|| {
    let state = AppState::init().unwrap();
    Arc::new(state)
});

#[tauri::command]
pub fn get_ui_config() -> Result<UiConfig, AppError> {
    Ok(APP_STATE.ui_config.lock().unwrap().clone())
}

pub fn get_wallet() -> Result<WalletCore, AppError> {
    Ok(APP_STATE.wallet.lock().unwrap().clone())
}

#[tauri::command]
pub fn set_ui_config_item(key: String, value: serde_json::Value) -> Result<(), AppError> {
    let mut ui_config = APP_STATE.ui_config.lock().unwrap();

    // 根据 key 更新对应的字段,将配置项保存到数据库
    match key.as_str() {
        "locale" => {
            if let Some(locale) = value.as_str() {
                ui_config.locale = Some(locale.to_string());

                config_set(key.clone(), value.clone())?;
            }
        }
        "dark_mode" => {
            if let Some(dark_mode) = value.as_bool() {
                ui_config.dark_mode = Some(dark_mode);
                config_set(key.clone(), value.clone())?;
            }
        }
        "current_account_index" => {
            if let Some(index) = value.as_u64() {
                ui_config.current_account_index = Some(index);
                config_set(key.clone(), value.clone())?;
            }
        }
        "auto_lock" => {
            if let Some(auto_lock) = value.as_bool() {
                ui_config.auto_lock = Some(auto_lock);
                config_set(key.clone(), value.clone())?;
            }
        }
        "auto_lock_timer" => {
            if let Some(timer) = value.as_u64() {
                ui_config.auto_lock_timer = Some(timer);
                config_set(key.clone(), value.clone())?;
            }
        }
        "currency" => {
            if let Some(currency) = value.as_str() {
                ui_config.currency = Some(currency.to_string());
                config_set(key.clone(), value.clone())?;
            }
        }
        "fiat" => {
            if let Some(fiat) = value.as_str() {
                ui_config.fiat = Some(fiat.to_string());
                config_set(key.clone(), value.clone())?;
            }
        }
        // 可以继续添加其他字段...
        _ => {
            // 对于未处理的字段，可以选择忽略或返回错误
            eprintln!("Unknown config key: {}", key);
        }
    }

    Ok(())
}

// ========== CONFIG ==========

pub fn make_config_key(field: &str) -> Vec<u8> {
    let mut key = Vec::new();
        key.extend_from_slice("config".as_bytes());
        key.push(b':');
        key.extend_from_slice(field.as_bytes());
        key
}


pub fn config_get(key: String) -> DbResult<Option<String>> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    mgr.get::<String>(&make_config_key(&key))
}

pub fn config_set(key: String, value: serde_json::Value) -> DbResult<()> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    // 将 serde_json::Value 序列化为 JSON 字符串进行存储
    let value_str = serde_json::to_string(&value).map_err(|e| AppError::JsonParseError(e))?;
    mgr.set(&make_config_key(&key), &value_str)
}


// reserve function for backup
pub fn config_batch_set(cfg: UiConfig) -> DbResult<()> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mut batch = WriteBatch::default();

    if let Some(v) = cfg.locale {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("locale"), &data);
    }
    if let Some(v) = cfg.dark_mode {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("dark_mode"), &data);
    }
    if let Some(v) = cfg.current_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("current_account_index"), &data);
    }
    if let Some(v) = cfg.next_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("next_account_index"), &data);
    }
    if let Some(v) = cfg.next_watch_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("next_watch_account_index"), &data);
    }
    if let Some(v) = cfg.next_airgap_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("next_airgap_account_index"), &data);
    }
    if let Some(v) = cfg.next_hdwallet_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("next_hdwallet_account_index"), &data);
    }
    if let Some(v) = cfg.auto_lock {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("auto_lock"), &data);
    }
    if let Some(v) = cfg.auto_lock_timer {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("auto_lock_timer"), &data);
    }
    if let Some(v) = cfg.active_apps {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("active_apps"), &data);
    }
    if let Some(v) = cfg.hidden_apps {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("hidden_apps"), &data);
    }
    if let Some(v) = cfg.currency {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("currency"), &data);
    }
    if let Some(v) = cfg.fiat {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("fiat"), &data);
    }
    if let Some(v) = cfg.is_initialized {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("is_initialized"), &data);
    }
    if let Some(v) = cfg.is_keystore_backuped {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("is_keystore_backuped"), &data);
    }

    db.write(&batch)
        .map_err(|e| AppError::DbWriteError(e.to_string()))
}

// reserve function for backup
pub fn config_batch_get() -> DbResult<UiConfig> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    let mut cfg = UiConfig::default();

    cfg.locale = mgr.get::<String>(&make_config_key("locale"))?;
    cfg.dark_mode = mgr.get::<bool>(&make_config_key("dark_mode"))?;
    cfg.current_account_index = mgr.get::<u64>(&make_config_key("current_account_index"))?;
    cfg.next_account_index = mgr.get::<u64>(&make_config_key("next_account_index"))?;
    cfg.next_watch_account_index = mgr.get::<u64>(&make_config_key("next_watch_account_index"))?;
    cfg.next_airgap_account_index = mgr.get::<u64>(&make_config_key("next_airgap_account_index"))?;
    cfg.next_hdwallet_account_index = mgr.get::<u64>(&make_config_key("next_hdwallet_account_index"))?;
    cfg.auto_lock = mgr.get::<bool>(&make_config_key("auto_lock"))?;
    cfg.auto_lock_timer = mgr.get::<u64>(&make_config_key("auto_lock_timer"))?;
    cfg.active_apps = mgr.get::<Vec<App>>(&make_config_key("active_apps"))?;
    cfg.hidden_apps = mgr.get::<Vec<App>>(&make_config_key("hidden_apps"))?;
    cfg.currency = mgr.get::<String>(&make_config_key("currency"))?;
    cfg.fiat = mgr.get::<String>(&make_config_key("fiat"))?;
    cfg.is_initialized = mgr.get::<bool>(&make_config_key("is_initialized"))?;
    cfg.is_keystore_backuped = mgr.get::<bool>(&make_config_key("is_keystore_backuped"))?;

    Ok(cfg)
}
