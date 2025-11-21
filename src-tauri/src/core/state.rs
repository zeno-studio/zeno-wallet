use std::sync::Arc;
use std::sync::Mutex;
use z_wallet_core::{WalletCore, constants};
use serde::{Deserialize, Serialize};
use crate::core::db::{
   AppDB, DbResult, TableKind, TableManager,
};
use crate::core::account::{account_list,VaultType};
use crate::core::vault::{vault_get};
use crate::error::AppError;
use crate::core::account::{Account};
use crate::data::addr::{AddressBookEntry, addressbook_list};
use rust_rocksdb::WriteBatch;
use tauri::State;
use bincode::{Decode, Encode};
use crate::apps::{Apps};
    
pub struct AppState {
    pub wallet: Arc<Mutex<WalletCore>>,
    pub persistent_config: Arc<Mutex<PersistentConfig>>,
    pub session_config: Arc<Mutex<SessionConfig>>,
    pub accounts: Arc<Mutex<Vec<Account>>>,
    pub address_books: Arc<Mutex<Vec<AddressBookEntry>>>,
    pub api_keys: Arc<Mutex<Vec<ApiKeyEntry>>>,
}

impl AppState {
    pub fn init(appdb: State<AppDB>) -> Result<AppState, AppError> {
        let mut wallet = WalletCore::default();
        let mut persistent_config = PersistentConfig::default();
        if let Ok(Some(init)) = config_get("is_initialized".to_string(), appdb.clone()) {
            if init == "true" {
                persistent_config = config_batch_get(appdb.clone())?;
                if let Some(vault) = vault_get(VaultType::V1.to_string(), appdb.clone()).unwrap() {
                    wallet = WalletCore {
                        vault,
                        derived_key: None,
                        expire_time: None,
                        cache_duration: Some(constants::DEFAULT_CACHE_DURATION),
                        entropy_bits: Some(constants::DEFAULT_ENTROPY_BITS),
                    };
                }
            }
        };
        let accounts = account_list(None, appdb.clone())?;
        let address_books = Vec::new();

        Ok(AppState {
            wallet: Arc::new(Mutex::new(wallet)),
            persistent_config: Arc::new(Mutex::new(persistent_config)),
            accounts: Arc::new(Mutex::new(accounts)),
            address_books: Arc::new(Mutex::new(address_books)),
            session_config: Arc::new(Mutex::new(SessionConfig::default())),
        })
    }
}



#[tauri::command]
pub fn get_persistent_config(state: State<AppState>) -> Result<PersistentConfig, AppError> {
    Ok(state.persistent_config.lock().unwrap().clone())
}

pub fn get_wallet(state: State<AppState>) -> Result<WalletCore, AppError> {
    Ok(state.wallet.lock().unwrap().clone())
}

pub fn get_accounts(state: State<AppState>) -> Result<Vec<Account>, AppError> {
    Ok(state.accounts.lock().unwrap().clone())
}
pub fn get_address_books(state: State<AppState>) -> Result<Vec<AddressBookEntry>, AppError> {
    Ok(state.address_books.lock().unwrap().clone())
}

pub fn get_current_chain(state: State<AppState>) -> Result<u64, AppError> {
    Ok(*state.current_chain.lock().unwrap())
}

