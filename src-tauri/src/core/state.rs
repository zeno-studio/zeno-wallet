use std::sync::Arc;
use std::sync::Mutex;
use z_wallet_core::{WalletCore, constants};
use serde::{Deserialize, Serialize};
use crate::core::db::{
   AppDB, DbResult, TableKind, TableManager,
};
use crate::core::account::{account_list,VaultType};
use crate::core::vault::{vault_get};
use crate::error::AppError;
use crate::core::account::{Account};
use crate::data::addr::{AddressBookEntry, addressbook_list};
use rust_rocksdb::WriteBatch;
use tauri::State;
use bincode::{Decode, Encode};
    
pub struct AppState {
    pub wallet: Arc<Mutex<WalletCore>>,
    pub persistent_config: Arc<Mutex<PersistentConfig>>,
    pub session_config: Arc<Mutex<SessionConfig>>,
    pub accounts: Arc<Mutex<Vec<Account>>>,
    pub address_books: Arc<Mutex<Vec<AddressBookEntry>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentConfig {
    pub locale: Option<String>,
    pub dark_mode: Option<bool>,
    pub current_account_address: Option<String>,
    pub current_account_index: Option<u64>,
    pub next_account_index: Option<u64>,
    pub next_watch_account_index: Option<u64>,
    pub next_airgap_account_index: Option<u64>,
    pub next_hdwallet_account_index: Option<u64>,
    pub wallet_lock_duration: Option<bool>,
    pub screen_lock_duration: Option<u64>,
    pub active_apps: Option<Vec<Apps>>,
    pub currency: Option<String>,
    pub fiat: Option<String>,
    pub is_initialized: Option<bool>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionConfig {
    pub current_account_index: Option<u64>,
    pub current_chain: Option<u64>,
    pub is_screen_locked: Option<bool>,
    pub is_wallet_locked: Option<bool>,
}

impl Default for PersistentConfig {
    fn default() -> Self {
        Self {
            locale: Some("en".to_string()),
            dark_mode: Some(false),
            current_account_index: Some(0),
            current_account_address: Some("0x0".to_string()),
            next_account_index: Some(1),
            next_watch_account_index: Some(101),
            next_airgap_account_index: Some(201),
            next_hdwallet_account_index: Some(301),
            wallet_lock_duration: Some(true),
            screen_lock_duration: Some(900), // in seconds
            active_apps: None,
            currency: Some("ETH".to_string()),
            fiat: Some("USD".to_string()),
            is_initialized: Some(false),
        }
    }
}

impl AppState {
    pub fn init(appdb: State<AppDB>) -> Result<AppState, AppError> {
        let mut wallet = WalletCore::default();
        let mut persistent_config = PersistentConfig::default();
        if let Ok(Some(init)) = config_get("is_initialized".to_string(), appdb.clone()) {
            if init == "true" {
                persistent_config = config_batch_get(appdb.clone())?;
                if let Some(vault) = vault_get(VaultType::V1.to_string(), appdb.clone()).unwrap() {
                    wallet = WalletCore {
                        vault,
                        derived_key: None,
                        expire_time: None,
                        cache_duration: Some(constants::DEFAULT_CACHE_DURATION),
                        entropy_bits: Some(constants::DEFAULT_ENTROPY_BITS),
                    };
                }
            }
        };
        let accounts = account_list(None, appdb.clone())?;
        let address_books = addressbook_list(None, appdb.clone())?;

        Ok(AppState {
            wallet: Arc::new(Mutex::new(wallet)),
            persistent_config: Arc::new(Mutex::new(persistent_config)),
            current_chain: Arc::new(Mutex::new(1)),
            accounts: Arc::new(Mutex::new(accounts)),
            address_books: Arc::new(Mutex::new(address_books)),
            is_locked: Arc::new(Mutex::new(true)),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub struct Apps {
    id: u64,
    name: String,
    app_path: String,
    description: String,
    supported_chain_id_ids: Vec<u64>,
}

#[tauri::command]
pub fn get_persistent_config(state: State<AppState>) -> Result<PersistentConfig, AppError> {
    Ok(state.persistent_config.lock().unwrap().clone())
}

pub fn get_wallet(state: State<AppState>) -> Result<WalletCore, AppError> {
    Ok(state.wallet.lock().unwrap().clone())
}

pub fn get_accounts(state: State<AppState>) -> Result<Vec<Account>, AppError> {
    Ok(state.accounts.lock().unwrap().clone())
}
pub fn get_address_books(state: State<AppState>) -> Result<Vec<AddressBookEntry>, AppError> {
    Ok(state.address_books.lock().unwrap().clone())
}

pub fn get_current_chain(state: State<AppState>) -> Result<u64, AppError> {
    Ok(*state.current_chain.lock().unwrap())
}

#[tauri::command]
pub fn set_persistent_config_item(
    key: String,
    value: serde_json::Value,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    let mut persistent_config = state.persistent_config.lock().unwrap();

    // 根据 key 更新对应的字段,将配置项保存到数据库
    match key.as_str() {
        "locale" => {
            if let Some(locale) = value.as_str() {
                persistent_config.locale = Some(locale.to_string());

                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "dark_mode" => {
            if let Some(dark_mode) = value.as_bool() {
                persistent_config.dark_mode = Some(dark_mode);
                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "current_account_index" => {
            if let Some(index) = value.as_u64() {
                persistent_config.current_account_index = Some(index);
                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "wallet_lock_duration" => {
            if let Some(wallet_lock_duration) = value.as_bool() {
                persistent_config.wallet_lock_duration = Some(wallet_lock_duration);
                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "screen_lock_duration" => {
            if let Some(timer) = value.as_u64() {
                persistent_config.screen_lock_duration = Some(timer);
                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "currency" => {
            if let Some(currency) = value.as_str() {
                persistent_config.currency = Some(currency.to_string());
                config_set(key.clone(), value.clone(), appdb)?;
            }
        }
        "fiat" => {
            if let Some(fiat) = value.as_str() {
                persistent_config.fiat = Some(fiat.to_string());
                config_set(key.clone(), value.clone(), appdb)?;
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

pub fn config_get(key: String, appdb: State<AppDB>) -> DbResult<Option<String>> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Config)?;
    mgr.get::<String>(&make_config_key(&key))
}

pub fn config_set(key: String, value: serde_json::Value, appdb: State<AppDB>) -> DbResult<()> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Config)?;
    // 将 serde_json::Value 序列化为 JSON 字符串进行存储
    let value_str = serde_json::to_string(&value).map_err(|e| AppError::JsonParseError(e))?;
    mgr.set(&make_config_key(&key), &value_str)
}

// reserve function for backup
pub fn config_batch_set(cfg: PersistentConfig, appdb: State<AppDB>) -> DbResult<()> {
    let db = appdb.db.as_ref();
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
    if let Some(v) = cfg.wallet_lock_duration {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("wallet_lock_duration"), &data);
    }
    if let Some(v) = cfg.screen_lock_duration {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("screen_lock_duration"), &data);
    }
    if let Some(v) = cfg.active_apps {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put(make_config_key("active_apps"), &data);
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

    db.write(&batch)
        .map_err(|e| AppError::DbWriteError(e.to_string()))
}

// reserve function for backup
pub fn config_batch_get(appdb: State<AppDB>) -> DbResult<PersistentConfig> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Config)?;
    let mut cfg = PersistentConfig::default();

    cfg.locale = mgr.get::<String>(&make_config_key("locale"))?;
    cfg.dark_mode = mgr.get::<bool>(&make_config_key("dark_mode"))?;
    cfg.current_account_index = mgr.get::<u64>(&make_config_key("current_account_index"))?;
    cfg.next_account_index = mgr.get::<u64>(&make_config_key("next_account_index"))?;
    cfg.next_watch_account_index = mgr.get::<u64>(&make_config_key("next_watch_account_index"))?;
    cfg.next_airgap_account_index =
        mgr.get::<u64>(&make_config_key("next_airgap_account_index"))?;
    cfg.next_hdwallet_account_index =
        mgr.get::<u64>(&make_config_key("next_hdwallet_account_index"))?;
    cfg.wallet_lock_duration = mgr.get::<bool>(&make_config_key("wallet_lock_duration"))?;
    cfg.screen_lock_duration = mgr.get::<u64>(&make_config_key("screen_lock_duration"))?;
    cfg.active_apps = mgr.get::<Vec<App>>(&make_config_key("active_apps"))?;
    cfg.currency = mgr.get::<String>(&make_config_key("currency"))?;
    cfg.fiat = mgr.get::<String>(&make_config_key("fiat"))?;
    cfg.is_initialized = mgr.get::<bool>(&make_config_key("is_initialized"))?;

    Ok(cfg)
}
