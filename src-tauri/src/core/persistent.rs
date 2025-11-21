
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistentConfig {
    pub is_initialized: Option<bool>,
    pub locale: Option<String>,
    pub dark_mode: Option<bool>,
    pub currency: Option<String>,
    pub fiat: Option<String>,
    pub next_account_index: Option<u64>,
    pub next_pq_account_index: Option<u64>,
    pub next_airgap_account_index: Option<u64>,
    pub next_hdwallet_account_index: Option<u64>,
    pub next_watch_account_index: Option<u64>,
    pub enable_screen_lock: Option<bool>,
    pub enable_biometric_auth: Option<bool>,
    pub screen_lock_duration: Option<u64>,
    pub wallet_lock_duration: Option<bool>,
    pub enable_ai_chat: Option<bool>,
    pub enable_ai_agent: Option<bool>,
    pub preferred_ai_provider: Option<String>,
    pub enable_notifications: Option<bool>,
    pub enable_auto_update: Option<bool>,
    pub enable_light_client: Option<bool>,
    pub preferred_rpc_mode: Option<RpcMode>,       // Custom / Ankr / Infura / Light client
    pub enable_browser_history: Option<bool>,
    pub enable_tx_history: Option<bool>,
    pub enable_smart_swap: Option<bool>,
    pub slippage_tolerance: Option<f32>,           // 默认 0.5%
    pub enable_anti_mev: Option<bool>,            // 自动用 Flashbots / Eden
    pub gas_price_multiplier: Option<f32>,         // 手动加价倍数（默认 1.2）
    pub enable_transaction_simulation: Option<bool>,  
}



impl Default for PersistentConfig {
    fn default() -> Self {
        Self {
            is_initialized: Some(false),
            locale: Some("en".to_string()),
            dark_mode: Some(false),
            currency: Some("ETH".to_string()),
            fiat: Some("USD".to_string()),
            next_account_index: Some(1),
            next_pq_account_index: Some(101),
            next_airgap_account_index: Some(201),
            next_hdwallet_account_index: Some(301),
            next_watch_account_index: Some(401),
            wallet_lock_duration: Some(true),
            screen_lock_duration: Some(900), // in seconds
        }
    }
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
