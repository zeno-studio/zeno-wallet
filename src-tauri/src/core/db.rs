use bincode::{Decode, Encode};
use rust_rocksdb::{
    ColumnFamilyDescriptor, DBWithThreadMode, Direction, IteratorMode, MultiThreaded, Options,
    SliceTransform, WriteBatch,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
// 引入自定义错误类型
use crate::data::tx::TransactionHistoryEntry;
use crate::error::AppError;

pub type DbResult<T> = Result<T, AppError>;

pub struct AppDB {
    pub db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl AppDB {
    /// 初始化 RocksDB，注册到 Tauri
    pub fn init(app_handle: &tauri::AppHandle) -> DbResult<Self> {
        // 1. 获取数据目录
        let app_dir = app_handle.path().app_data_dir().map_err(|e| {
            AppError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get app data directory: {}", e),
            ))
        })?;

        let path = app_dir.join("walletdb");
        std::fs::create_dir_all(&path).map_err(AppError::Io)?;

        // 2. 配置 Options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(8));

        // 3. 定义 Column Families
        let cf_names = vec![
            "default",
            TableKind::Config.as_str(),
            TableKind::Vault.as_str(),
            TableKind::Account.as_str(),
            TableKind::AddressBook.as_str(),
            TableKind::TxHistory.as_str(),
            TableKind::MsgHistory.as_str(),
        ];

        let cfs: Vec<_> = cf_names
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
            .collect();

        // 4. 打开数据库
        let db = DBWithThreadMode::<MultiThreaded>::open_cf_descriptors(&opts, &path, cfs)
            .map_err(|e| AppError::DbWriteError(e.to_string()))?;

        Ok(Self { db: Arc::new(db) })
    }
}

// ========== 表分类 ==========
#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq)]
pub enum TableKind {
    Config,
    Vault,
    Account,
    AddressBook,
    TxHistory,
    MsgHistory,
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
        }
    }
}
// ========== 数据结构定义 ==========

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

    pub fn key_from_str(&self, field: &str) -> Vec<u8> {
        let mut key = Vec::new();
        key.extend_from_slice(self.prefix.as_bytes());
        key.push(b':');
        key.extend_from_slice(&field.as_bytes());
        key
    }

    pub fn key_from_u64(&self, field: u64) -> Vec<u8> {
        let mut key = Vec::new();
        key.extend_from_slice(self.prefix.as_bytes());
        key.push(b':');
        key.extend_from_slice(&field.to_be_bytes());
        key
    }

    pub fn set<T: Serialize + bincode::Encode>(&self, field: &[u8], value: &T) -> DbResult<()> {
        let data = bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        self.db
            .put_cf(&self.cf, field, &data)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn get<T: Deserialize<'static> + bincode::Decode<()>>(
        &self,
        field: &[u8],
    ) -> DbResult<Option<T>> {
        match self.db.get_cf(&self.cf, field) {
            Ok(Some(data)) => {
                let result = bincode::decode_from_slice::<T, _>(&data, bincode::config::standard())
                    .map_err(|e| AppError::DbDeserializationError(e.to_string()))?;
                Ok(Some(result.0))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(AppError::DbReadError(e.to_string())),
        }
    }

    pub fn delete(&self, field: &[u8]) -> DbResult<()> {
        self.db
            .delete_cf(&self.cf, field)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn list<T: Deserialize<'static> + bincode::Decode<()>>(&self) -> DbResult<Vec<T>> {
        let mut items = Vec::new();
        // 创建二进制前缀
        let mut prefix_bytes = Vec::new();
        prefix_bytes.extend_from_slice(self.prefix.as_bytes());
        prefix_bytes.push(b':');

        let iter = self.db.prefix_iterator_cf(&self.cf, &prefix_bytes);
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

    fn make_key(chain_id: u64, timestamp: u64, hash: &str) -> Vec<u8> {
        let mut key = Vec::new();
        key.extend_from_slice("tx".as_bytes());
        key.push(b':');
        key.extend_from_slice(&chain_id.to_be_bytes());
        key.push(b':');
        key.extend_from_slice(&timestamp.to_be_bytes());
        key.push(b':');
        key.extend_from_slice(hash.as_bytes());
        key
    }

    pub fn insert(&self, item: &TransactionHistoryEntry) -> DbResult<()> {
        let key = Self::make_key(item.chain_id, item.timestamp, &item.tx_hash);
        let value = bincode::encode_to_vec(item, bincode::config::standard())
            .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
        self.db
            .put(key, value)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    pub fn range(
        &self,
        chain_id: u64,
        from: Option<u64>,
        to: Option<u64>,
    ) -> DbResult<Vec<TransactionHistoryEntry>> {
        let start = from.unwrap_or(0);
        let end = to.unwrap_or(u64::MAX);

        // 创建前缀 key 用于迭代
        let mut prefix = Vec::new();
        prefix.extend_from_slice("tx".as_bytes());
        prefix.push(b':');
        prefix.extend_from_slice(&chain_id.to_be_bytes());
        prefix.push(b':');

        // 创建起始和结束 key
        let mut start_key = prefix.clone();
        start_key.extend_from_slice(&start.to_be_bytes());

        let mut end_key = prefix.clone();
        end_key.extend_from_slice(&end.to_be_bytes());

        let iter = self
            .db
            .iterator(IteratorMode::From(&start_key, Direction::Forward));
        let mut result = Vec::new();

        for item in iter {
            match item {
                Ok((key, value)) => {
                    // 检查 key 是否在范围内且具有正确的前缀
                    if key.as_ref() > end_key.as_slice() || !key.starts_with(&prefix) {
                        break;
                    }
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(
                        &value,
                        bincode::config::standard(),
                    ) {
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

    pub fn find(&self, chain_id: u64, id: &str) -> DbResult<Option<TransactionHistoryEntry>> {
        // 创建前缀 key 用于迭代
        let mut prefix = Vec::new();
        prefix.extend_from_slice("tx".as_bytes());
        prefix.push(b':');
        prefix.extend_from_slice(&chain_id.to_be_bytes());
        prefix.push(b':');

        let iter = self.db.prefix_iterator(&prefix);

        for item in iter {
            match item {
                Ok((_key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(
                        &value,
                        bincode::config::standard(),
                    ) {
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

    pub fn delete(&self, chain_id: u64, id: &str) -> DbResult<()> {
        // 创建前缀 key 用于迭代
        let mut prefix = Vec::new();
        prefix.extend_from_slice("tx".as_bytes());
        prefix.push(b':');
        prefix.extend_from_slice(&chain_id.to_be_bytes());
        prefix.push(b':');

        let iter = self.db.prefix_iterator(&prefix);

        for item in iter {
            match item {
                Ok((key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(
                        &value,
                        bincode::config::standard(),
                    ) {
                        Ok((item, _)) => {
                            if item.tx_hash == id {
                                self.db
                                    .delete(&key)
                                    .map_err(|e| AppError::DbWriteError(e.to_string()))?;
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
            let key = Self::make_key(item.chain_id, item.timestamp, &item.tx_hash);
            let data = bincode::encode_to_vec(item, bincode::config::standard())
                .map_err(|e| AppError::DbSerializationError(e.to_string()))?;
            batch.put(key, &data);
        }
        self.db
            .write(&batch)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }

    /// 批量删除：根据 tx_hash 匹配
    pub fn batch_delete(&self, chain_id: u64, ids: &[String]) -> DbResult<()> {
        // 创建前缀 key 用于迭代
        let mut prefix = Vec::new();
        prefix.extend_from_slice("tx".as_bytes());
        prefix.push(b':');
        prefix.extend_from_slice(&chain_id.to_be_bytes());
        prefix.push(b':');

        let iter = self.db.prefix_iterator(&prefix);
        let mut batch = WriteBatch::default();

        for item in iter {
            match item {
                Ok((key, value)) => {
                    match bincode::decode_from_slice::<TransactionHistoryEntry, _>(
                        &value,
                        bincode::config::standard(),
                    ) {
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

        self.db
            .write(&batch)
            .map_err(|e| AppError::DbWriteError(e.to_string()))
    }
}
