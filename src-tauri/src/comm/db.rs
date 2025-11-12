use crate::api_registry::register_api;
use once_cell::sync::Lazy;
use revm::primitives::hardfork::SpecId::OSAKA;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DB, DBWithThreadMode, Direction, IteratorMode,
    MultiThreaded, Options, WriteBatch,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use tauri::State;
use z_wallet_core::Vault;

// ========== 全局 DB 实例 ==========
static DB_INSTANCE: Lazy<RwLock<Option<DBWithThreadMode<MultiThreaded>>>> =
    Lazy::new(|| RwLock::new(None));

// ========== 表分类 ==========
#[derive(Debug, Clone, Copy)]
pub enum TableKind {
    Config,
    Vault,
    Account,
    AddressBook,
    TxHistory,
    MessageHistory,
}

impl TableKind {
    fn as_str(&self) -> &'static str {
        match self {
            TableKind::Config => "config",
            TableKind::Vault => "vault",
            TableKind::Account => "account",
            TableKind::AddressBook => "addressbook",
            TableKind::TxHistory => "txhistory",
            TableKind::MessageHistory => "messagehistory",
        }
    }
}

// ========== 数据结构定义 ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub address: String,
    pub avatar: Option<String>, // emoji, 支持多字符
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: String,
    pub category: String,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// ========== 通用 Manager ==========
pub struct TableManager<'a> {
    db: &'a DB,
    cf: &'a ColumnFamily,
    prefix: &'static str,
}

impl<'a> TableManager<'a> {
    pub fn new(db: &'a DB, kind: TableKind) -> Result<Self, String> {
        let cf = db
            .cf_handle(kind.as_str())
            .ok_or("ColumnFamily not found")?;
        Ok(Self {
            db,
            cf,
            prefix: kind.as_str(),
        })
    }

    fn key(&self, field: &str) -> String {
        format!("{}:{}", self.prefix, field)
    }

    pub fn set<T: Serialize>(&self, field: &str, value: &T) -> Result<(), String> {
        let data = bincode::serialize(value).map_err(|e| e.to_string())?;
        self.db
            .put_cf(self.cf, self.key(field), data)
            .map_err(|e| e.to_string())
    }

    pub fn get<T: Deserialize<'static>>(&self, field: &str) -> Result<Option<T>, String> {
        match self.db.get_cf(self.cf, self.key(field)) {
            Ok(Some(data)) => Ok(Some(
                bincode::deserialize(&data).map_err(|e| e.to_string())?,
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn delete(&self, field: &str) -> Result<(), String> {
        self.db
            .delete_cf(self.cf, self.key(field))
            .map_err(|e| e.to_string())
    }

    pub fn list<T: Deserialize<'static>>(&self) -> Result<Vec<T>, String> {
        let mut items = Vec::new();
        let prefix = format!("{}:", self.prefix);
        let iter = self.db.prefix_iterator_cf(self.cf, prefix.as_bytes());
        for (key, value) in iter {
            if let Ok(item) = bincode::deserialize::<T>(&value) {
                items.push(item);
            }
        }
        Ok(items)
    }
    pub fn batch_set<T: Serialize>(&self, items: &[(String, T)]) -> Result<(), String> {
        let mut batch = WriteBatch::default();
        for (field, value) in items {
            let key = self.key(field);
            let data = bincode::serialize(value).map_err(|e| e.to_string())?;
            batch.put_cf(self.cf, key.as_bytes(), data);
        }
        self.db.write(batch).map_err(|e| e.to_string())
    }

    /// 批量删除：原子操作
    pub fn batch_delete(&self, fields: &[String]) -> Result<(), String> {
        let mut batch = WriteBatch::default();
        for field in fields {
            let key = self.key(field);
            batch.delete_cf(self.cf, key.as_bytes());
        }
        self.db.write(batch).map_err(|e| e.to_string())
    }
}

pub struct TxHistoryManager<'a> {
    db: &'a rocksdb::DB,
}

impl<'a> TxHistoryManager<'a> {
    pub fn new(db: &'a rocksdb::DB) -> Self {
        Self { db }
    }

    fn make_key(chain: &str, ts: u64, id: &str) -> String {
        format!("tx:{}:{:020}:{}", chain, ts, id)
    }

    pub fn insert(&self, item: &TransactionHistoryEntry) -> Result<(), String> {
        let key = Self::make_key(&item.chain, item.timestamp, &item.tx_hash);
        let value = bincode::serialize(item).map_err(|e| e.to_string())?;
        self.db.put(key, value).map_err(|e| e.to_string())
    }

    pub fn range(
        &self,
        chain: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<Vec<TransactionHistoryEntry>, String> {
        let start = from.unwrap_or(0);
        let end = to.unwrap_or(u64::MAX);
        let prefix = format!("tx:{}:", chain);
        let start_key = format!("{}{:020}", prefix, start);
        let end_key = format!("{}{:020}", prefix, end);

        let iter = self
            .db
            .iterator(IteratorMode::From(start_key.as_bytes(), Direction::Forward));
        let mut result = Vec::new();

        for (key, value) in iter {
            let k = key.map_err(|e| e.to_string())?;
            if k.as_ref() > end_key.as_bytes() || !k.starts_with(prefix.as_bytes()) {
                break;
            }
            let item: TransactionHistoryEntry =
                bincode::deserialize(&value.map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            result.push(item);
        }

        Ok(result)
    }

    pub fn find(&self, chain: &str, id: &str) -> Result<Option<TransactionHistoryEntry>, String> {
        let prefix = format!("tx:{}:", chain);
        let iter = self.db.prefix_iterator(prefix.as_bytes());

        for (_key, value) in iter {
            let item: TransactionHistoryEntry =
                bincode::deserialize(&value).map_err(|e| e.to_string())?;
            if item.tx_hash == id {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    pub fn delete(&self, chain: &str, id: &str) -> Result<(), String> {
        let iter = self.db.prefix_iterator(format!("tx:{}:", chain).as_bytes());
        for (key, value) in iter {
            let item: TransactionHistoryEntry =
                bincode::deserialize(&value).map_err(|e| e.to_string())?;
            if item.tx_hash == id {
                self.db.delete(key).map_err(|e| e.to_string())?;
                break;
            }
        }
        Ok(())
    }

    /// 批量插入：高效导入交易历史
    pub fn batch_insert(&self, items: &[TransactionHistoryEntry]) -> Result<(), String> {
        let mut batch = WriteBatch::default();
        for item in items {
            let key = Self::make_key(&item.chain, item.timestamp, &item.tx_hash);
            let data = bincode::serialize(item).map_err(|e| e.to_string())?;
            batch.put(key.as_bytes(), data);
        }
        self.db.write(batch).map_err(|e| e.to_string())
    }

    /// 批量删除：根据 tx_hash 匹配
    pub fn batch_delete(&self, chain: &str, ids: &[String]) -> Result<(), String> {
        let prefix = format!("tx:{}:", chain);
        let iter = self.db.prefix_iterator(prefix.as_bytes());
        let mut batch = WriteBatch::default();

        for (key, value) in iter {
            let value = value.map_err(|e| e.to_string())?;
            let item: TransactionHistoryEntry =
                bincode::deserialize(&value).map_err(|e| e.to_string())?;
            if ids.contains(&item.tx_hash) {
                batch.delete(key.map_err(|e| e.to_string())?);
            }
        }

        self.db.write(batch).map_err(|e| e.to_string())
    }
}

// ========== DB 初始化 ==========
#[tauri::command]
pub fn db_init(path: Option<String>) -> Result<(), String> {
    let path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("walletdb"));

    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_prefix_extractor(rocksdb::SliceTransform::create_fixed_prefix(8)); // 优化 prefix 读取

    let cf_names = vec![
        "default",
        TableKind::Config.as_str(),
        TableKind::Vault.as_str(),
        TableKind::Account.as_str(),
        TableKind::AddressBook.as_str(),
        TableKind::TxHistory.as_str(),
        TableKind::MessageHistory.as_str(),
    ];

    let cfs: Vec<_> = cf_names
        .iter()
        .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
        .collect();

    let db = DB::open_cf_descriptors(&opts, &path, cfs).map_err(|e| e.to_string())?;
    *DB_INSTANCE.write().unwrap() = Some(db);
    Ok(())
}

// ========== CONFIG ==========
#[tauri::command]
pub fn config_get(key: String) -> Result<Option<serde_json::Value>, String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    mgr.get(&key)
}

#[tauri::command]
pub fn config_set(key: String, value: serde_json::Value) -> Result<(), String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    mgr.set(&key, &value)
}
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BatchConfig {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub locale: Option<String>,
    pub dark_mode: Option<bool>,
    pub current_account_index: Option<u64>,
    pub next_account_index: Option<u64>,
    pub auto_lock: Option<bool>,
    pub auto_lock_timer: Option<u64>,
    pub active_apps: Option<Vec<App>>,
    pub hidden_apps: Option<Vec<App>>,
    pub currency: Option<String>,
    pub fiat: Option<String>,
}

#[tauri::command]
pub fn config_batch_set(cfg: BatchConfig) -> Result<(), String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Config)?;

    let mut batch = rocksdb::WriteBatch::default();

    // 针对每个 Option 字段存在的值，写入对应的 key
    if let Some(v) = cfg.theme {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:theme", data);
    }
    if let Some(v) = cfg.language {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:language", data);
    }
    if let Some(v) = cfg.locale {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:locale", data);
    }
    if let Some(v) = cfg.dark_mode {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:dark_mode", data);
    }
    if let Some(v) = cfg.current_account_index {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:current_account_index", data);
    }
    if let Some(v) = cfg.next_account_index {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:next_account_index", data);
    }
    if let Some(v) = cfg.auto_lock {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:auto_lock", data);
    }
    if let Some(v) = cfg.auto_lock_timer {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:auto_lock_timer", data);
    }
    if let Some(v) = cfg.active_apps {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:active_apps", data);
    }
    if let Some(v) = cfg.hidden_apps {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:hidden_apps", data);
    }
    if let Some(v) = cfg.currency {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:currency", data);
    }
    if let Some(v) = cfg.fiat {
        let data = bincode::serialize(&v).map_err(|e| e.to_string())?;
        batch.put("config:fiat", data);
    }

    db.write(batch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn config_batch_get() -> Result<BatchConfig, String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Config)?;
    let mut cfg = BatchConfig::default();
    cfg.theme = mgr.get("config:theme").transpose()?;
    cfg.language = mgr.get("config:language").transpose()?;
    cfg.locale = mgr.get("config:locale").transpose()?;
    cfg.dark_mode = mgr.get("config:dark_mode").transpose()?;
    cfg.current_account_index = mgr.get("config:current_account_index").transpose()?;
    cfg.next_account_index = mgr.get("config:next_account_index").transpose()?;
    cfg.auto_lock = mgr.get("config:auto_lock").transpose()?;
    cfg.auto_lock_timer = mgr.get("config:auto_lock_timer").transpose()?;
    cfg.active_apps = mgr.get("config:active_apps").transpose()?;
    cfg.hidden_apps = mgr.get("config:hidden_apps").transpose()?;
    cfg.currency = mgr.get("config:currency").transpose()?;
    cfg.fiat = mgr.get("config:fiat").transpose()?;
    Ok(cfg)
}

// ========== VAULT ==========
#[tauri::command]
pub fn vault_get(key: String) -> Result<Option<Vault>, String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Vault)?;
    mgr.get(&key)
}

#[tauri::command]
pub fn vault_set(key: String, v: Vault) -> Result<(), String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::Vault)?;
    mgr.set(&key, &v)
}

// ========== ADDRESSBOOK ==========
#[tauri::command]
pub fn addressbook_list(category: Option<String>) -> Result<Vec<AddressBookEntry>, String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let list = mgr.list::<AddressBookEntry>()?;
    Ok(match category {
        Some(cat) => list.into_iter().filter(|a| a.category == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn addressbook_add(entry: AddressBookEntry) -> Result<(), String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let key = format!("{}:{}", entry.category, entry.address);
    mgr.set(&key, &entry)
}
#[tauri::command]
pub fn addressbook_delete(category: String, address: String) -> Result<(), String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let key = format!("{}:{}", category, address);
    mgr.delete(&key)
}

#[tauri::command]
pub fn tx_list(
    chain: String,
    from: Option<u64>,
    to: Option<u64>,
) -> Result<Vec<TransactionHistoryEntry>, String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TxHistoryManager::new(db);
    mgr.range(&chain, from, to)
}

#[tauri::command]
pub fn tx_find(chain: String, id: String) -> Result<Option<TransactionHistoryEntry>, String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TxHistoryManager::new(db);
    mgr.find(&chain, &id)
}

#[tauri::command]
pub fn tx_delete(chain: String, id: String) -> Result<(), String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TxHistoryManager::new(db);
    mgr.delete(&chain, &id)
}

#[tauri::command]
pub fn tx_batch_insert(items: Vec<TransactionHistoryEntry>) -> Result<(), String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TxHistoryManager::new(db);
    mgr.batch_insert(&items)
}

#[tauri::command]
pub fn tx_batch_delete(chain: String, ids: Vec<String>) -> Result<(), String> {
    let guard = DB_INSTANCE.read().unwrap();
    let db = guard.as_ref().ok_or("DB not initialized")?;
    let mgr = TxHistoryManager::new(db);
    mgr.batch_delete(&chain, &ids)
}

// ========== MESSAGE HISTORY ==========
#[tauri::command]
pub fn message_add(chain: String, id: String, entry: MessageHistoryEntry) -> Result<(), String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::MessageHistory)?;
    let key = format!("{}:{}", chain, id);
    mgr.set(&key, &entry)
}

#[tauri::command]
pub fn message_list(chain: Option<String>) -> Result<Vec<MessageHistoryEntry>, String> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or("DB not initialized")?;
    let mgr = TableManager::new(db, TableKind::MessageHistory)?;
    let list = mgr.list::<MessageHistoryEntry>()?;
    Ok(match chain {
        Some(c) => list.into_iter().filter(|t| t.chain == c).collect(),
        None => list,
    })
}

// ========== 命令注册 ==========
pub fn register_all_commands(builder: tauri::Builder) -> tauri::Builder {
    builder.invoke_handler(tauri::generate_handler![
        // CONFIG
        config_get,
        config_set,
        config_batch_set,
        config_batch_get,
        // VAULT
        vault_get,
        vault_set,
        // ADDRESS BOOK
        addressbook_list,
        addressbook_add,
        addressbook_delete,
        // TX HISTORY
        tx_list,
        tx_find,
        tx_delete,
        tx_batch_insert,
        tx_batch_delete,
        // MESSAGE
        message_add,
        message_list,
        // DB INIT
        db_init
    ])
}
