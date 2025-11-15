use crate::core::db::{AppDB, TableKind, TableManager};
use crate::core::state::AppState;
use crate::error::AppError;
use tauri::State;

use z_wallet_core::{Vault, WalletCore, constants};

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

// ========== VAULT ==========
pub fn vault_get(key: String, appdb: State<AppDB>) -> Result<Option<Vault>, AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Vault)?;

    // 创建二进制 key
    let key = mgr.key_from_str(&key);

    // 获取存储的 JSON 字符串
    if let Some(vault_str) = mgr.get::<String>(&key)? {
        // 反序列化为 Vault 对象
        let vault: Vault =
            serde_json::from_str(&vault_str).map_err(|e| AppError::JsonParseError(e))?;
        Ok(Some(vault))
    } else {
        Ok(None)
    }
}

pub fn vault_add(key: String, vault: Vault, appdb: State<AppDB>) -> Result<(), AppError> {
    let db = appdb.db.as_ref();
    let mgr = TableManager::new(db, TableKind::Vault)?;

    // 创建二进制 key
    let key = mgr.key_from_str(&key);

    // 序列化为 JSON 字符串再存储
    let vault_str = serde_json::to_string(&vault).map_err(|e| AppError::JsonParseError(e))?;
    mgr.set(&key, &vault_str)?;
    Ok(())
}

#[tauri::command]
pub fn create_keystore(key: String, appdb: State<AppDB>) -> Result<String, AppError> {
    let key_clone = key.clone(); // Clone the key to avoid move issues
    if let Some(mut vault) = vault_get(key, appdb)? {
        return vault
            .to_keystore_string()
            .map_err(|e| AppError::WalletCoreError(e.to_string()));
    }
    Err(AppError::DbVaultNotFound(key_clone))
}

#[tauri::command]

pub fn import_account(
    key: String,
    keystore: String,
    password: String,
    appdb: State<AppDB>,
    state: State<AppState>,
) -> Result<(), AppError> {
    

}
