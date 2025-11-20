use serde::{Deserialize, Serialize};
use tauri::{ WebviewWindow};
use tauri::Emitter;

use tauri::State;
use crate::error::AppError;
use crate::core::state::{AppState, get_persistent_config};

#[derive(Debug, Deserialize)]
pub struct WebviewEthRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
    pub origin: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EthResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

// Example: request from webview -> rust via `window.__TAURI__.postMessage`
// Tauri will deliver message to the app as 'tauri://rpc' or you can implement a command to receive it.
// We'll implement commands to be invoked from the shell HTML as well.

#[tauri::command]
pub fn get_darkmode(state: State<AppState>) -> Result<bool,AppError> {
    let persistent_config = get_persistent_config(state)?;
    match persistent_config.dark_mode {
        Some(dark_mode) => Ok(dark_mode),
        None => Ok(false),
    }
}

#[tauri::command]
pub fn close_dapp_window(window: WebviewWindow) {
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
    let _ = app_handle.emit("SHELL_SHOW_MODAL", &payload);

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
            let _ = app_handle.emit("ETH_RESPONSE", &resp);
            Ok(sig)
        }
        Err(e) => {
            let resp = EthResponse {
                id: req.id,
                result: None,
                error: Some(serde_json::Value::String(e.clone())),
            };
            let _ = app_handle.emit("ETH_RESPONSE", &resp);
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
    // 根据不同平台使用不同的label
    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
    let label = "dapp-desktop";
    
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let label = "dapp-mobile";
    
    // 检查URL是否被允许
    if !is_allowed_url(&url) {
        return Err("URL not allowed".to_string());
    }

    let win = tauri::WebviewWindowBuilder::new(
        &app,
        label,
        tauri::WebviewUrl::External(url.parse().unwrap()),
    )
    .title("Dapp Browser")
    .build()
    .map_err(|e| e.to_string())?;

    // 发送事件到窗口，通知DApp已打开
    let _ = win.emit("dapp:open", serde_json::json!({
        "url": url,
        "title": "Dapp Browser"
    }));

    Ok(())
}

fn is_allowed_url(url: &str) -> bool {
    url.starts_with("https://") &&
    !url.contains("chrome-extension://") &&
    !url.contains("file://") &&
    !url.contains("tauri://")
}

// 添加获取当前账户地址的命令
// #[tauri::command]
// pub fn get_current_address() -> Result<String, String> {
//     let persistent_config = APP_STATE.persistent_config.lock().unwrap();
//     // 这里应该从实际的账户列表中获取当前账户地址
//     // 现在返回一个示例地址
//     Ok("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string())
// }

// // 添加获取链ID的命令
// #[tauri::command]
// pub fn get_chain_id() -> Result<String, String> {
//     let persistent_config = APP_STATE.persistent_config.lock().unwrap();
//     // 这里应该从实际的链配置中获取当前链ID
//     // 现在返回以太坊主网ID
//     Ok("0x1".to_string())
// }