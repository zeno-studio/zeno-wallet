use z_wallet_core::constants;

use crate::core::db::{Account, DB_INSTANCE, TableKind, TableManager, vault_add};
use crate::core::state::{APP_STATE, get_wallet, set_ui_config_item};
use crate::error::AppError;
use crate::utils::time;

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
pub fn account_list(account_type: Option<String>) -> Result<Vec<Account>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let list = mgr.list::<Account>()?;
    Ok(match account_type {
        Some(cat) => list.into_iter().filter(|a| a.account_type == cat).collect(),
        None => list,
    })
}

#[tauri::command]
pub fn account_add(index: u64, account: Account) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let mut key = mgr.key_from_u64(index);
    mgr.set(&key, &account)?;
    Ok(())
}

#[tauri::command]
pub fn account_get(index: u64) -> Result<Option<Account>, AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let mut key = mgr.key_from_u64(index);
    mgr.get::<Account>(&key)
}

#[tauri::command]
pub fn account_delete(index: u64) -> Result<(), AppError> {
    let db = DB_INSTANCE.read().unwrap();
    let db = db.as_ref().ok_or(AppError::DbNotInitialized)?;
    let mgr = TableManager::new(db, TableKind::Account)?;
    let mut key = mgr.key_from_u64(index);
    mgr.delete(&key)
}

#[tauri::command]
pub fn init_local_account(password: String) -> Result<(), AppError> {
    // 获取可写的 wallet 引用
    let mut wallet = get_wallet()?;

    // 创建 vault
    let (vault, address, path) = wallet
        .create_vault(
            &password,
            constants::DEFAULT_CACHE_DURATION,
            constants::DEFAULT_ENTROPY_BITS,
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
    vault_add(VaultType::V1.to_string(), vault)?;
    account_add(0u64, init_account)?;
    set_ui_config_item(
        "current_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1)),
    )?;
    set_ui_config_item(
        "next_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(2)),
    )?;
    set_ui_config_item("is_initialized".to_string(), serde_json::Value::Bool(true))?;

    Ok(())
}

#[tauri::command]
pub fn derive_local_account(password: String) -> Result<(), AppError> {
    let mut wallet = get_wallet()?;
    let index = APP_STATE
        .ui_config
        .lock()
        .unwrap()
        .next_account_index
        .unwrap();
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
    account_add(index, new_account)?;
    let next = APP_STATE
        .ui_config
        .lock()
        .unwrap()
        .next_account_index
        .unwrap()
        + 1;

    set_ui_config_item(
        "current_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(index)),
    )?;
    set_ui_config_item(
        "next_account_index".to_string(),
        serde_json::Value::Number(serde_json::Number::from(next)),
    )?;
    Ok(())
}

#[tauri::command]
pub fn hide_local_account(index: u64) -> Result<(), AppError> {
    let account = account_get(index)?;
    match account {
        Some(mut account) => {
            account.is_hidden = true;
            account_add(index, account)?;
            set_ui_config_item(
                "current_account_index".to_string(),
                serde_json::Value::Number(serde_json::Number::from(index)),
            )?;
        }
        None => return Err(AppError::DbAccountNotFound(index)),
    }

    Ok(())
}
