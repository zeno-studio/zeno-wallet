// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod db;
mod i18n;
mod utils;
use db::init_db;
use i18n::{I18nState, set_lang, t};
use serde::Deserialize;
use std::sync::Mutex;
use tauri::Manager;
use utils::password_check::{PasswordResult, check_password};

#[derive(Deserialize, Clone)]
struct WalletRequest {
    id: String,
    origin: String,
    method: String,
    params: serde_json::Value,
}

#[tauri::command]
async fn open_dapp(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::WebviewUrl;
    
    // Parse URL
    let parsed_url = url.parse::<url::Url>()
        .map_err(|e| format!("Invalid URL: {}", e))?;
    
    // Create DApp window with external URL
    let _dapp_window = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "dapp_window",
        WebviewUrl::External(parsed_url),
    )
    .title("DApp Browser")
    .inner_size(1024.0, 768.0)
    .build()
    .map_err(|e| format!("Failed to create DApp window: {}", e))?;
    
    // Create overlay toolbar window
    let _toolbar = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "dapp_toolbar",
        WebviewUrl::App("toolbar.html".into()),
    )
    .title("Toolbar")
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .build()
    .map_err(|e| format!("Failed to create toolbar: {}", e))?;
    
    Ok(())
}

#[tauri::command]
fn show_sign_modal(app: tauri::AppHandle, payload: String) {
    use tauri::WebviewUrl;
    
    // Create sign modal window
    let _modal = tauri::WebviewWindowBuilder::new(
        &app,
        "sign_modal",
        WebviewUrl::App("sign_modal.html".into()),
    )
    .title("Sign Request")
    .inner_size(400.0, 300.0)
    .build();
    
    // TODO: Emit event to modal with payload
    // In Tauri 2.x, event emission API may need to be updated
    // For now, window is created and payload can be passed via URL or other means
    let _ = _modal;
    let _ = payload;
}
// This command is invoked by preload to forward wallet requests
#[tauri::command]
fn wallet_request(app: tauri::AppHandle, req: WalletRequest) -> Result<serde_json::Value, String> {
    use tauri::WebviewUrl;
    
    // 1) basic origin validation (additional checks advisable)
    println!("wallet request from {} method {}", req.origin, req.method);

    // 2) quick sanity checks: method whitelist, param size limit, etc.
    // e.g., allow only "eth_sendTransaction", "eth_sign", "personal_sign" etc.

    // 3) open a modal window to request user confirmation (local file sign_modal.html)
    let modal_id = format!("sign_modal_{}", req.id);
    let modal = tauri::WebviewWindowBuilder::new(
        &app,
        &modal_id,
        WebviewUrl::App("sign_modal.html".into()),
    )
    .title("Confirm Transaction")
    .inner_size(400.0, 300.0)
    .build()
    .map_err(|e| format!("Failed to create modal: {}", e))?;

    // Pass request data to modal
    // TODO: In Tauri 2.x, event emission API may need to be updated
    // For now, window is created and data can be passed via URL parameters or other means
    // Optionally store request in a HashMap keyed by req.id for modal to retrieve
    let _ = modal;

    // Wait for an event from modal (sign:result) with same id
    // TODO: In real implementation, use oneshot channel or HashMap to store responder
    // and wait for modal result with timeout
    // For now, return error placeholder
    Err("not implemented: await modal result and return signature".into())
}

#[tauri::command]
fn check_password_cmd(pw: String) -> PasswordResult {
    check_password(&pw)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .manage(Mutex::new(I18nState::new("en"))) // 默认语言
        .setup(|app| {
            init_db(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_password_cmd,
            set_lang,
            t,
            open_dapp,
            wallet_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
