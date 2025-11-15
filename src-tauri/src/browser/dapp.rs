use serde::{Deserialize, Serialize};
use tauri::{Manager, Window};
use std::sync::Arc;
use crate::core::state::APP_STATE;

#[derive(Debug, Deserialize)]
pub struct WebviewEthRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
    pub origin: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EthResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

// Example: request from webview -> rust via `window.__TAURI__.postMessage`
// Tauri will deliver message to the app as 'tauri://rpc' or you can implement a command to receive it.
// We'll implement commands to be invoked from the shell HTML as well.

#[tauri::command]
pub fn get_darkmode() -> bool {
    let darkmode = APP_STATE.ui_config.lock().unwrap().dark_mode;
    match darkmode {
        Some(dark_mode) => dark_mode,
        None => false,
    };
    false
}

#[tauri::command]
pub fn close_dapp_window(window: Window) {
    let _ = window.close();
}

#[tauri::command]
pub async fn get_balance() -> Result<String, String> {
    // 示例：你实际应通过 RPC 查询余额
    Ok("0.1234 ETH".to_string())
}

#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub id: u64,
    pub origin: Option<String>,
    pub method: String,
    pub params: serde_json::Value,
    pub details: Option<String>,
}

// Simulated signing function - replace with real key management
fn simulate_sign(_payload: &SignRequest) -> Result<String, String> {
    // NEVER do this for production. Replace with HSM, secure enclave, or your wallet signing flow.
    Ok("0xSIMULATED_SIGNATURE".to_string())
}

#[tauri::command]
pub async fn sign_transaction(
    app_handle: tauri::AppHandle,
    req: SignRequest
) -> Result<String, String> {
    // 1. permission/origin check (very important)
    if let Some(origin) = &req.origin {
        // implement your whitelist logic, e.g. check saved allowed origins
        let allowed = true; // dummy
        if !allowed {
            return Err(format!("origin {} not allowed", origin));
        }
    }

    // 2. show confirmation modal in the shell window (emit event)
    // We emit an event "SHELL_SHOW_MODAL" to the specific dapp window; the shell listens and shows UI
    let payload = serde_json::json!({
        "title": "签名请求",
        "details": req.details.clone().unwrap_or_else(|| req.method.clone()),
        "origin": req.origin.clone().unwrap_or_default(),
        "reqId": req.id,
        "data": req.params,
    });
    // Assuming your dapp window label is "dapp"
    let _ = app_handle.emit_all("SHELL_SHOW_MODAL", payload);

    // In production you'd block / await user confirmation via another command or event
    // For demo, we simulate immediate user confirmation:
    // (Better approach: send event, then wait on a short-lived channel triggered when user clicks confirm)
    // We'll perform a direct sign for this demo:
    match simulate_sign(&req) {
        Ok(sig) => {
            // after signing, emit an ETH_RESPONSE back to webview context
            let resp = EthResponse {
                id: req.id,
                result: Some(serde_json::Value::String(sig.clone())),
                error: None,
            };
            // backend uses Window::emit to forward result to the shell which will forward into the webview
            let _ = app_handle.emit_all("ETH_RESPONSE", resp);
            Ok(sig)
        }
        Err(e) => {
            let resp = EthResponse {
                id: req.id,
                result: None,
                error: Some(serde_json::Value::String(e.clone())),
            };
            let _ = app_handle.emit_all("ETH_RESPONSE", resp);
            Err(e)
        }
    }
}

// Example command to handle forwarded messages from shell (if shell posts to Rust)
#[tauri::command]
pub fn dapp_post_message(payload: serde_json::Value) -> Result<(), String> {
    // this receives arbitrary JSON posted from the shell page
    // route or validate accordingly
    println!("dapp_post_message: {}", payload);
    Ok(())
}

#[tauri::command]
pub async fn open_dapp_window(
    app: tauri::AppHandle,
    url: String,
) -> Result<(), String> {
    let label = format!("dapp-{}", uuid::Uuid::new_v4());

    let win = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::External(url.parse().unwrap()),
    )
    .title("Dapp Browser")
    .build()
    .map_err(|e| e.to_string())?;

    let mut ui = APP_STATE.ui_config.lock().await;

    let session = DappSession {
        window_label: label.clone(),
        origin: url.clone(),
        provider: ProviderConfig {
            chain_id: ui.current_chain_id,
            accounts: ui.current_account.clone().into_iter().collect(),
            selected_address: ui.current_account.clone(),
        }
    };

    ui.dapp_sessions.insert(label.clone(), session.clone());

    win.emit("wallet:provider-config", &session.provider)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn is_allowed_url(url: &str) -> bool {
    url.starts_with("https://") &&
    !url.contains("chrome-extension://") &&
    !url.contains("file://") &&
    !url.contains("tauri://")
}

