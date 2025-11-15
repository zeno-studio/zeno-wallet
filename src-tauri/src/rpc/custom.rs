use serde::{Deserialize, Serialize};
use tauri::State;
use bincode::{Decode, Encode};
use crate::core::db::{AppDB, TableKind, TableManager};
use crate::error::AppError;


#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct CustomRpc {
    pub chain_id: u64,
    pub endpoint: String,
    pub protocol: String,
}

#[tauri::command]
pub fn custom_rpc_list(appdb: State<AppDB>) -> Result<Vec<CustomRpc>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    mgr.list::<CustomRpc>()
}

#[tauri::command]
pub fn custom_rpc_add(rpc: CustomRpc, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    let mut key = mgr.key_from_u64(rpc.chain_id);
    key.push(b':');
    key.extend_from_slice(&rpc.protocol.as_bytes());

    mgr.set(&key, &rpc)
}

#[tauri::command]
pub fn custom_rpc_delete(
    chain_id: u64,
    protocol: String,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::CustomRpc)?;
    let mut key = mgr.key_from_u64(chain_id);
    key.push(b':');
    key.extend_from_slice(&protocol.as_bytes());
    mgr.delete(&key)
}
