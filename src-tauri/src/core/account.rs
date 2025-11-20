use crate::core::db::{AppDB, TableKind, TableManager};
use crate::core::state::{AppState, config_get, get_wallet, set_persistent_config_item};
use crate::core::vault::vault_add;

use crate::error::AppError;
use crate::utils::time;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tauri::State;
use z_wallet_core::{Vault, constants};

#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode, PartialEq)]
pub struct Account {
    pub name: String,
    pub address: String,
    pub account_type: String, // local | watch
    pub account_index: u64,
    pub derive_path: String,
    pub avatar: Option<String>, // emoji, 支持多字
    pub memo: Option<String>,
    pub ens: Option<String>,
    pub nft: Option<String>, // {chain_idid ,address,tokenid}
    pub created_at: u64,
    pub is_hidden: bool,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            address: "".to_string(),
            account_type: "local".to_string(),
            account_index: 0,
            derive_path: "".to_string(),
            avatar: None,
            memo: None,
            ens: None,
            nft: None,
            created_at: 0,
            is_hidden: false,
        }
    }
}

pub enum VaultType {
    V1,
}
impl VaultType {
    pub fn to_string(&self) -> String {
        match self {
            VaultType::V1 => constants::VERSION_TAG_1.to_string(),
        }
    }
}

pub enum AccountType {
    Local,
    Hardware,
    Airgap,
    Watch,
}

impl AccountType {
    pub fn to_string(&self) -> String {
        match self {
            AccountType::Local => "local".to_string(),
            AccountType::Hardware => "hardware".to_string(),
            AccountType::Airgap => "airgap".to_string(),
            AccountType::Watch => "watch".to_string(),
        }
    }
}

// ========== ACCOUNT ==========
#[tauri::command]
pub fn account_list(
    account_type: Option<String>,
    appdb: State<AppDB>,
) -> Result<Vec<Account>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Account)?;
    let list = mgr.list::<Account>()?;
    Ok(match account_type {
        Some(cat) => list.into_iter().filter(|a| a.account_type == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn account_add(index: u64, account: Account, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Account)?;
    let key = mgr.key_from_u64(index);
    mgr.set(&key, &account)?;
    Ok(())
}

#[tauri::command]
pub fn account_get(index: u64, appdb: State<AppDB>) -> Result<Option<Account>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Account)?;
    let key = mgr.key_from_u64(index);
    mgr.get::<Account>(&key)
}

#[tauri::command]
pub fn account_delete(index: u64, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Account)?;
    let key = mgr.key_from_u64(index);
    mgr.delete(&key)
}

#[tauri::command]
pub fn init_local_account(
    password: String,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    // 获取可写的 wallet 引用
    if let Ok(Some(init)) = config_get("is_initialized".to_string(), appdb.clone()) {
        if init == "true" {
            return Err(AppError::AlreadyInitialized);
        }
    }

    let mut wallet = get_wallet(state.clone())?;

    // 创建 vault
    let (vault, address, path) = wallet
        .create_vault(
            &password,
            constants::DEFAULT_ENTROPY_BITS,
            Some(constants::DEFAULT_CACHE_DURATION),
            time::now_s(),
        )
        .map_err(|e| AppError::WalletCoreError(e.to_string()))?;

    let init_account = Account {
        name: format!("Account {}", 0),
        address: address,
        account_type: AccountType::Local.to_string(),
        account_index: 0,
        derive_path: path,
        created_at: time::now_s(),
        ..Default::default()
    };

    // 保存到数据库
    vault_add(VaultType::V1.to_string(), vault, appdb.clone())?;
    account_add(0u64, init_account, appdb.clone())?;
    set_persistent_config_item(
        "current_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1)),
        appdb.clone(),
        state.clone(),
    )?;
    set_persistent_config_item(
        "next_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(2)),
        appdb.clone(),
        state.clone(),
    )?;
    set_persistent_config_item(
        "is_initialized".to_string(),
        serde_json::Value::Bool(true),
        appdb,
        state,
    )?;

    Ok(())
}

#[tauri::command]
pub fn derive_local_account(
    password: String,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    let mut wallet = get_wallet(state.clone())?;
    let index = state.persistent_config.lock().unwrap().next_account_index.unwrap();
    let (address, path) = wallet
        .derive_account(&password, index as u32, time::now_s())
        .map_err(|e| AppError::WalletCoreError(e.to_string()))?;

    let new_account = Account {
        name: format!("Account {}", index),
        address: address,
        account_type: AccountType::Local.to_string(),
        account_index: index,
        derive_path: path,
        created_at: time::now_s(),
        ..Default::default()
    };

    // 保存到数据库
    account_add(index, new_account, appdb.clone())?;
    let next = state.persistent_config.lock().unwrap().next_account_index.unwrap() + 1;

    set_persistent_config_item(
        "current_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(index)),
        appdb.clone(),
        state.clone(),
    )?;
    set_persistent_config_item(
        "next_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(next)),
        appdb,
        state,
    )?;
    Ok(())
}

#[tauri::command]
pub fn hide_local_account(
    index: u64,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    let account = account_get(index, appdb.clone())?;
    match account {
        Some(mut account) => {
            account.is_hidden = true;
            account_add(index, account, appdb.clone())?;
            set_persistent_config_item(
                "current_account_index".to_string(),
                serde_json::Value::Number(serde_json::Number::from(index)),
                appdb,
                state,
            )?;
        }
        None => return Err(AppError::DbAccountNotFound(index)),
    }

    Ok(())
}

pub fn import_account(
    keystore: String,
    password: String,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    if let Ok(Some(init)) = config_get("is_initialized".to_string(), appdb.clone()) {
        if init == "true" {
            return Err(AppError::AlreadyInitialized);
        }
    }
    let vault = Vault::from_keystore_string(&keystore)
        .map_err(|e| AppError::WalletCoreError(e.to_string()))?;
    vault
        .verify_password(&password)
        .map_err(|e| AppError::WalletCoreError(e.to_string()))?;
    let mut wallet = get_wallet(state.clone())?;

    // Clone the vault before it's moved to import_vault
    let vault_for_storage = vault.clone();
    let (address, path) = wallet
        .import_vault(
            &password,
            vault,
            Some(constants::DEFAULT_CACHE_DURATION),
            time::now_s(),
        )
        .map_err(|e| AppError::WalletCoreError(e.to_string()))?;
    let init_account = Account {
        name: format!("Account {}", 0),
        address: address,
        account_type: AccountType::Local.to_string(),
        account_index: 0,
        derive_path: path,
        created_at: time::now_s(),
        ..Default::default()
    };

    // 保存到数据库
    vault_add(VaultType::V1.to_string(), vault_for_storage, appdb.clone())?;
    account_add(0u64, init_account, appdb.clone())?;
    set_persistent_config_item(
        "current_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1)),
        appdb.clone(),
        state.clone(),
    )?;
    set_persistent_config_item(
        "next_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(2)),
        appdb.clone(),
        state.clone(),
    )?;
    set_persistent_config_item(
        "is_initialized".to_string(),
        serde_json::Value::Bool(true),
        appdb,
        state,
    )?;

    Ok(())
}
