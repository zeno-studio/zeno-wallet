use std::sync::Arc;
use tokio::sync::Mutex; 
use z_wallet_core::{WalletCore, constants};
use serde::{Deserialize, Serialize};
use rust_rocksdb::WriteBatch;
use tauri::State;
use bincode::{Decode, Encode};
use reqwest::Client;
use crate::core::db::{
   AppDB, DbResult, TableKind, TableManager,
};
use crate::core::account::{Account,account_list,};
use crate::core::vault::{VaultType,vault_get};
use crate::core::persistent::{PersistentConfig, SessionConfig, config_get, config_batch_get};
use crate::core::session::{SessionConfig};
use crate::data::addr::{AddressBookEntry, addressbook_list};
use crate::data::nft::Nft;
use crate::data::token::Token;
use crate::rpc::https::create_https_client;
use crate::helios::client::{HeliosClient, init_helios};
use crate::ai::provider::{AiProvider};
use crate::rpc::gateway::{GatewayManager};
use crate::evm::chain::Chain;
use crate::apps::{Apps};
use crate::error::AppError;


pub struct AppState {
    pub wallet: Arc<Mutex<WalletCore>>,
    pub https_client: Arc<Mutex<Client>>,
    pub helios_client: Arc<Mutex<HeliosClient>>,
    pub gateway_manager: Arc<Mutex<GatewayManager>>,
    pub user_tokens: Arc<Mutex<Option<Vec<Token>>>>,
    pub user_nfts: Arc<Mutex<Option<Vec<Nft>>>>,
    pub active_dapp_host: Arc<Mutex<Option<String>>>, 

    //sync to js
    pub config: Arc<Mutex<Config>>,
    pub accounts: Arc<Mutex<Vec<Account>>>,
    pub address_books: Arc<Mutex<Vec<AddressBookEntry>>>,
    pub ai_providers: Arc<Mutex<Option<Vec<AiProvider>>>>,
    pub current_account_index: Arc<Mutex<Option<u64>>>,
    pub current_chain_id: Arc<Mutex<Option<u64>>>,
    pub helios_current_chain_id: Arc<Mutex<Option<u64>>>,
    pub is_screen_locked: Arc<Mutex<Option<bool>>>,
    pub is_wallet_locked: Arc<Mutex<Option<bool>>>,

}

impl AppState {
    pub fn init(appdb: State<AppDB>) -> Result<AppState, AppError> {
        let mut wallet = WalletCore::default();
        let mut config = PersistentConfig::default();
        if let Ok(Some(init)) = config_get("is_initialized".to_string(), appdb.clone()) {
            if init == "true" {
                config = config_batch_get(appdb.clone())?;
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
        
        // 初始化 Helios 客户端
        let helios_client = tauri::async_runtime::block_on(async {
            match init_helios().await {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Failed to initialize Helios client: {}", e);
                    // 返回默认客户端或panic，取决于需求
                    panic!("Failed to initialize Helios client: {}", e);
                }
            }
        });

        Ok(AppState {
            wallet: Arc::new(Mutex::new(wallet)),

            https_client: Arc::new(Mutex::new(create_https_client())),  
            helios_client: Arc::new(Mutex::new(helios_client)),
            gateway_manager: Arc::new(Mutex::new(GatewayManager::default())),
            user_tokens: Arc::new(Mutex::new(None)),
            user_nfts: Arc::new(Mutex::new(None)),
            active_dapp_host: Arc::new(Mutex::new(None)),

            config: Arc::new(Mutex::new(config)),
            ai_providers: Arc::new(Mutex::new(None)),
            accounts: Arc::new(Mutex::new(accounts)),
            address_books: Arc::new(Mutex::new(address_books)),
            current_account_index: Arc::new(Mutex::new(None)),
            current_chain_id: Arc::new(Mutex::new(None)),
            helios_current_chain_id: Arc::new(Mutex::new(None)),
            is_screen_locked: Arc::new(Mutex::new(None)),
            is_wallet_locked: Arc::new(Mutex::new(None)),
            
        })
    }
}



#[tauri::command]
pub fn get_config(state: State<AppState>) -> Result<PersistentConfig, AppError> {
    Ok(state.config.blocking_lock().clone())
}
#[tauri::command]
pub fn get_accounts(state: State<AppState>) -> Result<Vec<Account>, AppError> {
    Ok(state.accounts.blocking_lock().clone())
}
#[tauri::command]
pub fn get_address_books(state: State<AppState>) -> Result<Vec<AddressBookEntry>, AppError> {
    Ok(state.address_books.blocking_lock().clone())
}
#[tauri::command]
pub fn get_current_chain_id(state: State<AppState>) -> Result<u64, AppError> {
    Ok(state.current_chain_id.blocking_lock())
}

#[tauri::command]
pub fn get_current_account_index(state: State<AppState>) -> Result<u64, AppError> {
    Ok(state.current_account_index.blocking_lock())
}

#[tauri::command]
pub fn get_is_screen_locked(state: State<AppState>) -> Result<bool, AppError> {
    Ok(state.is_screen_locked.blocking_lock().unwrap_or(false))
}

#[tauri::command]
pub fn get_is_wallet_locked(state: State<AppState>) -> Result<bool, AppError> {
    Ok(state.is_wallet_locked.blocking_lock().unwrap_or(false))
}

