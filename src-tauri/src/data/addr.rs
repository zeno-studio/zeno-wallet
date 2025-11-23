use crate::core::db::{AppDB, TableKind, TableManager};
use crate::error::AppError;
use alloy_primitives::Address;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: Address,
    pub category: String,
    pub memo: Option<String>,
    pub chain_ids: Option<Vec<u64>>, // not eoa
}

impl AddressBookEntry {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            address: Address::zero(),
            category: "Contact".to_string(),
            memo: None,
            chain_ids: None,
        }
    }
}

// ========== ADDRESSBOOK ==========
#[tauri::command]
pub fn addressbook_list(
    category: Option<String>,
    appdb: State<AppDB>,
) -> Result<Vec<AddressBookEntry>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    let list = mgr.list::<AddressBookEntry>()?;
    Ok(match category {
        Some(cat) => list.into_iter().filter(|a| a.category == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn addressbook_add(entry: AddressBookEntry, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    // 创建二进制 key
    let key = mgr.key_from_str(&entry.address);
    mgr.set(&key, &entry)
}

#[tauri::command]
pub fn addressbook_delete(address: String, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::AddressBook)?;
    // 创建二进制 key
    let key = mgr.key_from_str(&address);
    mgr.delete(&key)
}

