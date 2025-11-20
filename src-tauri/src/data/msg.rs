use serde::{Deserialize, Serialize};
use tauri::State;
use bincode::{Decode, Encode};
use crate::core::db::{AppDB, TableKind, TableManager};


#[derive(Debug, Serialize, Deserialize, Clone,Encode, Decode, PartialEq)]
pub struct MessageHistoryEntry {
    pub chain_id: u64,
    pub msg_type: String, // "191" | "712"
    pub signer: String,
    pub msg_hash: String,
    pub payload: Option<String>,
    pub signature: Option<String>,
    pub timestamp: u64,
    pub status: Option<String>,
}
// 引入自定义错误类型
use crate::error::AppError;
#[tauri::command]
pub fn message_add(
    chain_id: u64,
    hash: String,
    entry: MessageHistoryEntry,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let mut key = mgr.key_from_u64(chain_id);
    key.push(b':');
    key.extend_from_slice(&hash.as_bytes());
    let entry_str = serde_json::to_string(&entry).map_err(AppError::JsonParseError)?;
    mgr.set(&key, &entry_str)
}

#[tauri::command]
pub fn message_delete(chain_id: u64, hash: String, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let mut key = Vec::new();
    key.extend_from_slice(&chain_id.to_be_bytes());
    key.push(b':');
    key.extend_from_slice(&hash.as_bytes());
    // 创建二进制 key
    mgr.delete(&key)
}

#[tauri::command]
pub fn message_list(
    chain_id: Option<u64>,
    appdb: State<AppDB>,
) -> Result<Vec<MessageHistoryEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::MsgHistory)?;
    let list = mgr.list::<String>()?; // 存储的是JSON字符串
    let mut result = Vec::new();
    for item_str in list {
        match serde_json::from_str::<MessageHistoryEntry>(&item_str) {
            Ok(item) => {
                if let Some(ref c) = chain_id {
                    if item.chain_id == *c {
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
