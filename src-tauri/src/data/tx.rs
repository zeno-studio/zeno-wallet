use serde::{Deserialize, Serialize};
use tauri::State;
use bincode::{Decode, Encode};
use crate::core::db::{AppDB,TxHistoryManager};
use crate::error::AppError;
use alloy_primitives::{U256, U128};


#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct TransactionHistoryEntry {
    pub chain_id: u64,
    pub hash: String,
    pub block_number: u64,
    pub from: String,
    pub to: String,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub gas_used: Option<U256>,
    pub timestamp: Option<u64>,
    pub status: Option<String>,
}
pub trait IntoInterTx{
    fn into_inter(self) -> TransactionHistoryEntry;
}


// ========== Transaction History ==========
#[tauri::command]
pub fn tx_list(
    chain_id: u64,
    from: Option<u64>,
    to: Option<u64>,
    appdb: State<AppDB>,
) -> Result<Vec<TransactionHistoryEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.range(chain_id, from, to)
}

#[tauri::command]
pub fn tx_add(entry: TransactionHistoryEntry, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.insert(&entry)
}

#[tauri::command]
pub fn tx_find(
    chain_id: u64,
    hash: String,
    appdb: State<AppDB>,
) -> Result<Option<TransactionHistoryEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.find(chain_id, &hash)
}

#[tauri::command]
pub fn tx_delete(chain_id: u64, hash: String, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.delete(chain_id, &hash)
}

#[tauri::command]
pub fn tx_batch_insert(
    items: Vec<TransactionHistoryEntry>,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.batch_insert(&items)
}

#[tauri::command]
pub fn tx_batch_delete(
    chain_id: u64,
    hashs: Vec<String>,
    appdb: State<AppDB>,
) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TxHistoryManager::new(db);
    mgr.batch_delete(chain_id, &hashs)
}
