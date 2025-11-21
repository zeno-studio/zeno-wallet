use serde::{Deserialize, Serialize};
use tauri::State;
use bincode::{Decode, Encode};
use crate::core::db::{AppDB, TableKind, TableManager};
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct AddressBookEntry {
    pub name: String,
    pub address: String,
    pub category: String,
    pub memo: Option<String>,
}

impl AddressBookEntry {
    pub fn new(name: String, address: String, category: String, memo: Option<String>) -> Self {
        Self { name, address, category, memo }
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

