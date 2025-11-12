use z_wallet_core::{WalletCore, constants};
use tauri::State;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::sync::Arc;

use crate::comm::db::{UiConfig, db_init, config_batch_set, config_get,config_set,config_batch_get,vault_get, vault_set};
use crate::error::AppError;

pub struct AppState {
    pub wallet: RwLock<WalletCore>,
    pub ui_config: RwLock<UiConfig>,
}

pub enum VaultType {
    Local,
    Backup,
}
impl VaultType {
    pub fn to_string(&self) -> String {
        match self {
            VaultType::Local => "local".to_string(),
            VaultType::Backup => "backup".to_string(),
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
                if let Some(vault) = vault_get(VaultType::Local.to_string()).unwrap() {
                    wallet = WalletCore{
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
            wallet: RwLock::new(wallet),
            ui_config: RwLock::new(ui_config),
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
    Ok(APP_STATE.ui_config.read().unwrap().clone())
}

pub fn get_wallet() -> Result<WalletCore, AppError> {
    Ok(APP_STATE.wallet.read().unwrap().clone())
}

#[tauri::command]
pub fn set_ui_config_item(key: String, value: serde_json::Value) -> Result<(), AppError> {
    // 将配置项保存到数据库
    config_set(key.clone(), value.clone())?;
    
    // 更新内存中的配置
    let mut ui_config = APP_STATE.ui_config.write().unwrap();
    
    // 根据 key 更新对应的字段
    match key.as_str() {
        "locale" => {
            if let Some(locale) = value.as_str() {
                ui_config.locale = Some(locale.to_string());
            }
        },
        "dark_mode" => {
            if let Some(dark_mode) = value.as_bool() {
                ui_config.dark_mode = Some(dark_mode);
            }
        },
        "current_account_index" => {
            if let Some(index) = value.as_u64() {
                ui_config.current_account_index = Some(index);
            }
        },
        "auto_lock" => {
            if let Some(auto_lock) = value.as_bool() {
                ui_config.auto_lock = Some(auto_lock);
            }
        },
        "auto_lock_timer" => {
            if let Some(timer) = value.as_u64() {
                ui_config.auto_lock_timer = Some(timer);
            }
        },
        "currency" => {
            if let Some(currency) = value.as_str() {
                ui_config.currency = Some(currency.to_string());
            }
        },
        "fiat" => {
            if let Some(fiat) = value.as_str() {
                ui_config.fiat = Some(fiat.to_string());
            }
        },
        // 可以继续添加其他字段...
        _ => {
            // 对于未处理的字段，可以选择忽略或返回错误
            eprintln!("Unknown config key: {}", key);
        }
    }
    
    Ok(())
}


#[tauri::command]
pub fn init_account(password: String, timestamp: u64) -> Result<(), AppError> {
    // 获取可写的 wallet 引用
    let mut wallet = APP_STATE.wallet.write().unwrap();
    
    // 创建 vault
    let (vault, _address, _path) = wallet.create_vault(
        &password,
        constants::DEFAULT_CACHE_DURATION,
        constants::DEFAULT_ENTROPY_BITS,
        timestamp,
    ).map_err(|e| AppError::WalletCoreError(e.to_string()))?;
    
    // 保存到数据库
    vault_set(VaultType::Local.to_string(), vault)?;

    set_ui_config_item("current_account_index".to_string(), serde_json::Value::Number(serde_json::Number::from(1)))?;
    set_ui_config_item("next_account_index".to_string(), serde_json::Value::Number(serde_json::Number::from(2)))?;
    set_ui_config_item("is_initialized".to_string(), serde_json::Value::Bool(true))?; 
    
    Ok(())
}

