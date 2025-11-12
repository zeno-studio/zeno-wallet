use once_cell::sync::Lazy;
use rust_rocksdb::{
    ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode,
    MultiThreaded, Options, WriteBatch,
};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::sync::Arc;
use tauri::State;
use z_wallet_core::{Vault,};
use bincode::{Decode, Encode};

// 引入Tauri插件相关模块和路径API
use tauri::{Manager};

// 引入自定义错误类型
use crate::error::{AppError, DbResult};

// ========== 全局 DB 实例 ==========
pub static DB_INSTANCE: Lazy<RwLock<Option<DBWithThreadMode<MultiThreaded>>>> =
    Lazy::new(|| RwLock::new(None));

// ========== 表分类 ==========
#[derive(Debug, Clone, Copy,Encode, Decode, PartialEq)]
pub enum TableKind {
    Config,
    Vault,
    Account,
    AddressBook,
    TxHistory,
    MsgHistory,
    CustomRpc,
}

impl TableKind {
    fn as_str(&self) -> &'static str {
        match self {
            TableKind::Config => "config",
            TableKind::Vault => "vault",
            TableKind::Account => "account",
            TableKind::AddressBook => "addressbook",
            TableKind::TxHistory => "txhistory",
            TableKind::MsgHistory => "msghistory",
            TableKind::CustomRpc => "customrpc",
        }
    }
}
// ========== 数据结构定义 ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub locale: Option<String>,
    pub dark_mode: Option<bool>,
    pub current_account_index: Option<u64>,
    pub next_account_index: Option<u64>,
    pub next_watch_account_index: Option<u64>,
    pub next_airgap_account_index: Option<u64>,
    pub next_hdwallet_account_index: Option<u64>,
    pub auto_lock: Option<bool>,
    pub auto_lock_timer: Option<u64>,
    pub active_apps: Option<Vec<App>>,
    pub hidden_apps: Option<Vec<App>>,
    pub currency: Option<String>,
    pub fiat: Option<String>,
    pub is_initialized: Option<bool>,
    pub is_keystore_backuped: Option<bool>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            locale: Some("en".to_string()),
            dark_mode: Some(false),
            current_account_index: Some(0),
            next_account_index: Some(1),
            next_watch_account_index: Some(101),
            next_airgap_account_index: Some(201),
            next_hdwallet_account_index: Some(301),
            auto_lock: Some(true),
            auto_lock_timer: Some(900), // in seconds
            active_apps: None,
            hidden_apps: None,
            currency: Some("ETH".to_string()),
            fiat: Some("USD".to_string()),
            is_initialized: Some(false),
            is_keystore_backuped: Some(false),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone,Encode, Decode, PartialEq)]
pub struct Account{
    pub name: String,
    pub address: String,
    pub account_type: String, // local | watch
    pub account_index: u32,
    pub derive_path: String,
    pub avatar: Option<String>, // emoji, 支持多字
    pub memo: Option<String>,
    pub ens: Option<String>,
    pub nft: Option<String>,  // {chainid ,address,tokenid}
    pub created_at: u64,
    pub is_hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone,Encode, Decode, PartialEq)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: String,
    pub category: String,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone,Encode, Decode, PartialEq)]
pub struct TransactionHistoryEntry {
    pub chain: String,
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: u64,
    pub status: Option<String>,
    pub raw: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MessageHistoryEntry {
    pub chain: String,
    pub msg_type: String, // "191" | "712"
    pub signer: String,
    pub msg_hash: String,
    pub payload: serde_json::Value,
    pub signature: Option<String>,
    pub timestamp: u64,
    pub status: Option<String>,
}

// 为App结构实现Encode和Decode trait
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
pub struct App {
    id: u64,
    name: String,
    app_path: String,
    description: String,
    supported_chain_ids: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone,Encode, Decode, PartialEq)]
pub struct CustomRpc {
    pub chain: String,
    pub rpc_type: String,
    pub endpoint: String,
}

// ========== 通用 Manager ==========
pub struct TableManager<'a> {
    db: &'a DBWithThreadMode<MultiThreaded>,
    cf: Arc<rust_rocksdb::BoundColumnFamily<'a>>,
    prefix: &'static str,
}

impl<'a> TableManager<'a> {
    pub fn new(db: &'a DBWithThreadMode<MultiThreaded>, kind: TableKind) -> DbResult<Self> {
        let cf = db
            .cf_handle(kind.as_str())
            .ok_or(AppError::DbColumnFamilyNotFound)?;
        Ok(Self {
            db,
            cf,
            prefix: kind.as_str(),
        })
    }

    fn key(&self, field: &str) -> String {
        format!("{}:{}", self.prefix, field)
    }

    pub fn set<T: Serialize + bincode::Encode>(&self, field: &str, value: &T) -> DbResult<()> {
        let data = bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        self.db
            .put_cf(&self.cf, self.key(field), data)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn get<T: Deserialize<'static> + bincode::Decode<()>>(&self, field: &str) -> DbResult<Option<T>> {
        match self.db.get_cf(&self.cf, self.key(field)) {
            Ok(Some(data)) => {
                let result = bincode::decode_from_slice::<T, _>(&data, bincode::config::standard())
                    .map_err(|e| AppError::DbDeserializationError(e.to_string()))?;
                Ok(Some(result.0))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::DbReadError(e.to_string())),
        }
    }

    pub fn delete(&self, field: &str) -> DbResult<()> {
        self.db
            .delete_cf(&self.cf, self.key(field))
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn list<T: Deserialize<'static> + bincode::Decode<()>>(&self) -> DbResult<Vec<T>> {
        let mut items = Vec::new();
        let prefix = format!("{}:", self.prefix);
        let iter = self.db.prefix_iterator_cf(&self.cf, prefix.as_bytes());
        for item in iter {
            match item {
                Ok((_key, value)) => {
                    match bincode::decode_from_slice::<T, _>(&value, bincode::config::standard()) {
                        Ok((item, _)) => items.push(item),
                        Err(e) => {
                            eprintln!("Failed to decode item: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Iterator error: {}", e);
                    continue;
                }
            }
        }
        Ok(items)
    }
}

pub struct TxHistoryManager<'a> {
    db: &'a DBWithThreadMode<MultiThreaded>,
}

impl<'a> TxHistoryManager<'a> {
    pub fn new(db: &'a DBWithThreadMode<MultiThreaded>) -> Self {
        Self { db }
    }

    fn make_key(chain: &str, ts: u64, id: &str) -> String {
        format!("tx:{}:{:020}:{}", chain, ts, id)
    }

    pub fn insert(&self, item: &TransactionHistoryEntry) -> DbResult<()> {
        let key = Self::make_key(&item.chain, item.timestamp, &item.tx_hash);
        let value = bincode::encode_to_vec(item, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        self.db.put(key, value).map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn range(
        &self,
        chain: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> DbResult<Vec<TransactionHistoryEntry>> {
        let start = from.unwrap_or(0);
        let end = to.unwrap_or(u64::MAX);
        let prefix = format!("tx:{}:", chain);
        let start_key = format!("{}{:020}", prefix, start);
        let end_key = format!("{}{:020}", prefix, end);

        let iter = self
            .db
            .iterator(IteratorMode::From(start_key.as_bytes(), Direction::Forward));
        let mut result = Vec::new();

        for item in iter {
            match item {
                Ok((key, value)) => {
                    if key.as_ref() > end_key.as_bytes() || !key.starts_with(prefix.as_bytes()) {
                        break;
                    }
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(&value, bincode::config::standard()) {
                        Ok((item, _)) => result.push(item),
                        Err(e) => {
                            eprintln!("Failed to decode transaction history entry: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Iterator error: {}", e);
                    continue;
                }
            }
        }

        Ok(result)
    }

    pub fn find(&self, chain: &str, id: &str) -> DbResult<Option<TransactionHistoryEntry>> {
        let prefix = format!("tx:{}:", chain);
        let iter = self.db.prefix_iterator(prefix.as_bytes());

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(&value, bincode::config::standard()) {
                        Ok((item, _)) => {
                            if item.tx_hash == id {
                                return Ok(Some(item));
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to decode transaction history entry: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Iterator error: {}", e);
                    continue;
                }
            }
        }
        Ok(None)
    }

    pub fn delete(&self, chain: &str, id: &str) -> DbResult<()> {
        let prefix = format!("tx:{}:", chain);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        
        for item in iter {
            match item {
                Ok((key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(&value, bincode::config::standard()) {
                        Ok((item, _)) => {
                            if item.tx_hash == id {
                                self.db.delete(&key).map_err(|e| AppError::DbWriteError(e.to_string()))?;
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to decode transaction history entry: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Iterator error: {}", e);
                    continue;
                }
            }
        }
        Ok(())
    }

    /// 批量插入：高效导入交易历史
    pub fn batch_insert(&self, items: &[TransactionHistoryEntry]) -> DbResult<()> {
        let mut batch = WriteBatch::default();
        for item in items {
            let key = Self::make_key(&item.chain, item.timestamp, &item.tx_hash);
            let data = bincode::encode_to_vec(item, bincode::config::standard())
                .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
            batch.put(key.as_bytes(), &data);
        }
        self.db.write(&batch).map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    /// 批量删除：根据 tx_hash 匹配
    pub fn batch_delete(&self, chain: &str, ids: &[String]) -> DbResult<()> {
        let prefix = format!("tx:{}:", chain);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        let mut batch = WriteBatch::default();

        for item in iter {
            match item {
                Ok((key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(&value, bincode::config::standard()) {
                        Ok((item, _)) => {
                            if ids.contains(&item.tx_hash) {
                                batch.delete(&key);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to decode transaction history entry: {}", e);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Iterator error: {}", e);
                    continue;
                }
            }
        }

        self.db.write(&batch).map_err(|e| AppError::DbWriteError(e.to_string()))
    }
}

// ========== VAULT ==========
#[tauri::command]
pub fn vault_get(key: String) -> Result<Option<Vault>, AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Vault)?;
    
    // 获取存储的 JSON 字符串
    if let Some(vault_str) = mgr.get::<String>(&key)? {
        // 反序列化为 Vault 对象
        let vault: Vault = serde_json::from_str(&vault_str)
            .map_err(|e| AppError::JsonParseError(e))?;
        Ok(Some(vault))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn vault_set(key: String, vault: Vault) -> Result<(), AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Vault)?;
    
    // 序列化为 JSON 字符串再存储
    let vault_str = serde_json::to_string(&vault)
        .map_err(|e| AppError::JsonParseError(e))?;
    mgr.set(&key, &vault_str)
}

// ========== ACCOUNT ==========
#[tauri::command]
pub fn account_list(category: Option<String>) -> Result<Vec<Account>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let list = mgr.list::<Account>()?;
    Ok(match category {
        Some(cat) => list.into_iter().filter(|a| a.account_type == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn account_add(account: Account) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let key = account.address.clone();
    mgr.set(&key, &account)
}

#[tauri::command]
pub fn account_delete(address: String) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    mgr.delete(&address)
}
// ========== ADDRESSBOOK ==========
#[tauri::command]
pub fn addressbook_list(category: Option<String>) -> Result<Vec<AddressBookEntry>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let list = mgr.list::<AddressBookEntry>()?;
    Ok(match category {
        Some(cat) => list.into_iter().filter(|a| a.category == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn addressbook_add(entry: AddressBookEntry) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let key = format!("{}:{}", entry.category, entry.address);
    mgr.set(&key, &entry)
}

#[tauri::command]
pub fn addressbook_delete(category: String, address: String) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let key = format!("{}:{}", category, address);
    mgr.delete(&key)
}

#[tauri::command]
pub fn tx_list(
    chain: String,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<TransactionHistoryEntry>, AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.range(&chain, from, to)
}

// ========== CUSTOM RPC ==========
#[tauri::command]
pub fn custom_rpc_list() -> Result<Vec<CustomRpc>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    mgr.list::<CustomRpc>()
}

#[tauri::command]
pub fn custom_rpc_add(rpc: CustomRpc) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    let key = format!("{}", rpc.chain);
    mgr.set(&key, &rpc)
}

#[tauri::command]
pub fn custom_rpc_delete(chain_id: u64) -> Result<(), AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    mgr.delete(&chain_id.to_string())
}


// ========== MESSAGE HISTORY ==========
#[tauri::command]
pub fn tx_add(entry: TransactionHistoryEntry) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.insert(&entry)
}

#[tauri::command]
pub fn tx_find(chain: String, id: String) -> Result<Option<TransactionHistoryEntry>, AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.find(&chain, &id)
}

#[tauri::command]
pub fn tx_delete(chain: String, id: String) -> Result<(), AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.delete(&chain, &id)
}

#[tauri::command]
pub fn tx_batch_insert(items: Vec<TransactionHistoryEntry>) -> Result<(), AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.batch_insert(&items)
}

#[tauri::command]
pub fn tx_batch_delete(chain: String, ids: Vec<String>) -> Result<(), AppError> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TxHistoryManager::new(db);
    mgr.batch_delete(&chain, &ids)
}

// ========== MESSAGE HISTORY ==========
#[tauri::command]
pub fn message_add(chain: String, id: String, entry: MessageHistoryEntry) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let key = format!("{}:{}", chain, id);
    let entry_str = serde_json::to_string(&entry).map_err(AppError::JsonParseError)?;
    mgr.set(&key, &entry_str)
}

#[tauri::command]
pub fn message_delete(chain: String, id: String) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let key = format!("{}:{}", chain, id);
    mgr.delete(&key)
}

#[tauri::command]
pub fn message_list(chain: Option<String>) -> Result<Vec<MessageHistoryEntry>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let list = mgr.list::<String>()?; // 存储的是JSON字符串
    let mut result = Vec::new();
    for item_str in list {
        match serde_json::from_str::<MessageHistoryEntry>(&item_str) {
            Ok(item) => {
                if let Some(ref c) = chain {
                    if item.chain == *c {
                        result.push(item);
                    }
                } else {
                    result.push(item);
                }
            }
            Err(e) => {
                // 跳过无法解析的项，记录错误但不中断
                eprintln!("Failed to parse message history entry: {}", e);
                continue;
            }
        }
    }
    Ok(result)
}

// ========== CONFIG ==========

pub fn config_get(key: String) -> DbResult<Option<String>> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    mgr.get::<String>(&key)
}

pub fn config_set(key: String, value: serde_json::Value) -> DbResult<()> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    let value_str = serde_json::to_string(&value).map_err(|e| AppError::JsonParseError(e))?;
    mgr.set(&key, &value_str)
}

pub fn config_batch_set(cfg: UiConfig) -> DbResult<()> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mut batch = WriteBatch::default();

    if let Some(v) = cfg.locale {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:locale", &data);
    }
    if let Some(v) = cfg.dark_mode {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:dark_mode", &data);
    }
    if let Some(v) = cfg.current_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:current_account_index", &data);
    }
    if let Some(v) = cfg.next_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:next_account_index", &data);
    }
    if let Some(v) = cfg.next_watch_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:next_watch_account_index", &data);
    }
    if let Some(v) = cfg.next_airgap_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:next_airgap_account_index", &data);
    }
    if let Some(v) = cfg.next_hdwallet_account_index {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:next_hdwallet_account_index", &data);
    }
    if let Some(v) = cfg.auto_lock {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:auto_lock", &data);
    }
    if let Some(v) = cfg.auto_lock_timer {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:auto_lock_timer", &data);
    }
    if let Some(v) = cfg.active_apps {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:active_apps", &data);
    }
    if let Some(v) = cfg.hidden_apps {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:hidden_apps", &data);
    }
    if let Some(v) = cfg.currency {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:currency", &data);
    }
    if let Some(v) = cfg.fiat {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:fiat", &data);
    }
    if let Some(v) = cfg.is_initialized {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:is_initialized", &data);
    }
    if let Some(v) = cfg.is_keystore_backuped {
        let data = bincode::encode_to_vec(&v, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        batch.put("config:is_keystore_backuped", &data);
    }

    db.write(&batch).map_err(|e| AppError::DbWriteError(e.to_string()))
}

pub fn config_batch_get() -> DbResult<UiConfig> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    let mut cfg = UiConfig::default();
    
    cfg.locale = mgr.get::<String>("locale")?;
    cfg.dark_mode = mgr.get::<bool>("dark_mode")?;
    cfg.current_account_index = mgr.get::<u64>("current_account_index")?;
    cfg.next_account_index = mgr.get::<u64>("next_account_index")?;
    cfg.next_watch_account_index = mgr.get::<u64>("next_watch_account_index")?;
    cfg.next_airgap_account_index = mgr.get::<u64>("next_airgap_account_index")?;
    cfg.next_hdwallet_account_index = mgr.get::<u64>("next_hdwallet_account_index")?;   
    cfg.auto_lock = mgr.get::<bool>("auto_lock")?;
    cfg.auto_lock_timer = mgr.get::<u64>("auto_lock_timer")?;
    cfg.active_apps = mgr.get::<Vec<App>>("active_apps")?;
    cfg.hidden_apps = mgr.get::<Vec<App>>("hidden_apps")?;
    cfg.currency = mgr.get::<String>("currency")?;
    cfg.fiat = mgr.get::<String>("fiat")?;
    cfg.is_initialized = mgr.get::<bool>("is_initialized")?;
    cfg.is_keystore_backuped = mgr.get::<bool>("is_keystore_backuped")?;
    
    Ok(cfg)
}

// ========== DB 初始化 ==========
pub fn db_init(app_handle: &tauri::AppHandle) -> DbResult<()> {
    // 使用 Tauri 2 的 app dir API
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to get app data directory: {}", e))))?;
    let path = app_dir.join("walletdb");

    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_prefix_extractor(rust_rocksdb::SliceTransform::create_fixed_prefix(8)); // 优化 prefix 读取

    let cf_names = vec![
        "default",
        TableKind::Config.as_str(),
        TableKind::Vault.as_str(),
        TableKind::Account.as_str(),
        TableKind::AddressBook.as_str(),
        TableKind::TxHistory.as_str(),
        TableKind::MsgHistory.as_str(),
        TableKind::CustomRpc.as_str(),
    ];

    let cfs: Vec<_> = cf_names
        .iter()
        .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
        .collect();

    let db = DBWithThreadMode::<MultiThreaded>::open_cf_descriptors(&opts, &path, cfs)
        .map_err(|e| AppError::DbWriteError(e.to_string()))?;
    *DB_INSTANCE.write().unwrap() = Some(db);
    Ok(())
}
