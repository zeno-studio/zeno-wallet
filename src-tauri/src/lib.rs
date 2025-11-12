// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod comm;
mod error;
mod constants;

use std::sync::Mutex;
use comm::i18n::I18nState;
use comm::webview::{open_dapp, wallet_request};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(comm::db::register_all_commands)
        .manage(Mutex::new(I18nState::new("en"))) // 默认语言
        .setup(|app| {
            init_db(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comm::password::check_password,
            comm::i18n::set_lang,
            comm::i18n::t,
            comm::qr::scan_qr,
            open_dapp,
            wallet_request
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
