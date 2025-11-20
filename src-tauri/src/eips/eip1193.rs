use tauri::{Manager, Window};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EthRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}


#[tauri::command]
pub async fn ethereum_request(
    window: tauri::Window,
    method: String,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {

    match method.as_str() {
        "eth_chainId" => chain_id().await,
        "eth_accounts" => accounts().await,
        "eth_requestAccounts" => request_accounts(window).await,
        "personal_sign" => personal_sign(window, params).await,
        "eth_signTypedData_v4" => sign_typed_data(window, params).await,
        "eth_sendTransaction" => send_tx(window, params).await,
        _ => Err(format!("Unsupported method: {}", method)),
    }
}


async fn chain_id() -> Result<serde_json::Value, String> {
    let ui = APP_STATE.persistent_config.lock().await;
    Ok(format!("0x{:x}", ui.current_chain_id).into())
}

async fn accounts() -> Result<serde_json::Value, String> {
    let ui = APP_STATE.persistent_config.lock().await;
    let acc = ui.current_account.clone().map(|a| vec![a]).unwrap_or(vec![]);
    Ok(serde_json::json!(acc))
}

async fn request_accounts(
    window: tauri::Window,
) -> Result<serde_json::Value, String> {

    let request_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();

    {
        let mut ui = APP_STATE.persistent_config.lock().await;
        ui.pending_requests.insert(request_id.clone(), PendingRequest { sender: tx });
    }

    window.emit("wallet:request-accounts", request_id.clone())
        .map_err(|e| e.to_string())?;

    let approved = rx.await.map_err(|_| "channel closed")?;

    Ok(approved)
}

#[tauri::command]
pub async fn approve_request(id: String, result: serde_json::Value) -> Result<(), String> {
    let mut ui = APP_STATE.persistent_config.lock().await;

    if let Some(p) = ui.pending_requests.remove(&id) {
        let _ = p.sender.send(result);
    }
    Ok(())
}

async fn sign_typed_data(
    window: tauri::Window,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {

    let (address, typed_data) = parse_typed_params(params)?;

    let sig = {
        let wallet = APP_STATE.wallet.lock().await;
        wallet.sign_eip712(address, typed_data)?
    };

    Ok(sig.into())
}

#[tauri::command]
pub async fn switch_chain(window: tauri::Window, chain_id: u64) -> Result<(), String> {
    {
        let mut ui = APP_STATE.persistent_config.lock().await;
        ui.current_chain_id = chain_id;

        for (_, session) in ui.dapp_sessions.iter() {
            let win = window.app_handle().get_webview_window(&session.window_label).unwrap();
            win.emit("chainChanged", format!("0x{:x}", chain_id)).ok();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn select_account(window: tauri::Window, account: String) -> Result<(), String> {
    {
        let mut ui = APP_STATE.persistent_config.lock().await;
        ui.current_account = Some(account.clone());

        for (_, session) in ui.dapp_sessions.iter() {
            let win = window.app_handle().get_webview_window(&session.window_label).unwrap();
            win.emit("accountsChanged", vec![account.clone()]).ok();
        }
    }
    Ok(())
}

// Add function to create EIP-6963 provider detail for Zeno Wallet
pub fn create_zeno_wallet_provider_detail() -> EIP6963ProviderDetail {
    EIP6963ProviderDetail {
        info: EIP6963ProviderInfo {
            uuid: format!("zeno-wallet-{}", uuid::Uuid::new_v4()),
            name: "Zeno Wallet".to_string(),
            icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjI0IiBoZWlnaHQ9IjI0IiBmaWxsPSJ3aGl0ZSIvPgo8cGF0aCBkPSJNMTIgMkM2LjQ4IDAgMiA0LjQ4IDIgMTBDMiAxNS41MiA2LjQ4IDIwIDEyIDIwQzE3LjUyIDIwIDIyIDE1LjUyIDIyIDEwQzIyIDQuNDggMTcuNTIgMCAxMiAyWk0xMCAxN0w1IDEyTDEwIDdMMTAgMTJaTTE0IDE3TDE5IDEyTDE0IDdMMTQgMTJaIiBmaWxsPSIjMzMzMzMzIi8+Cjwvc3ZnPg==".to_string(),
            rdns: "com.zenowallet".to_string(),
        }
    }
}

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use std::collections::HashMap;
use tauri::Window;
use uuid;

#[derive(Clone, Debug, Serialize)]
pub struct ProviderConfig {
    pub chain_id: u64,
    pub accounts: Vec<String>,
    pub selected_address: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DappSession {
    pub window_label: String,
    pub origin: String,
    pub provider: ProviderConfig,
}

pub struct PendingRequest {
    pub sender: oneshot::Sender<serde_json::Value>,
}

// Add EIP-6963 provider info structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP6963ProviderInfo {
    pub uuid: String,
    pub name: String,
    pub icon: String,
    pub rdns: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EIP6963ProviderDetail {
    pub info: EIP6963ProviderInfo,
}