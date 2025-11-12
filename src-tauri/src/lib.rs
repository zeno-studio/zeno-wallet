// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod comm;
mod error;
mod constants;
mod state;

use std::sync::Mutex;
use comm::i18n::I18nState;
use comm::webview::{open_dapp, wallet_request};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            comm::db::db_init(&app.handle()).unwrap();
            state::AppState::tauri_setup(&app.handle()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comm::password::check_password_strength,
            comm::i18n::set_lang,
            comm::i18n::t,
            comm::qr::scan_qr,
            open_dapp,
            wallet_request,
            // DB 相关命令
            comm::db::vault_get,
            comm::db::vault_set,
            comm::db::account_list,
            comm::db::account_add,
            comm::db::account_delete,
            comm::db::addressbook_list,
            comm::db::addressbook_add,
            comm::db::addressbook_delete,
            comm::db::tx_list,
            comm::db::custom_rpc_list,
            comm::db::custom_rpc_add,
            comm::db::custom_rpc_delete,
            comm::db::tx_add,
            comm::db::tx_find,
            comm::db::tx_delete,
            comm::db::tx_batch_insert,
            comm::db::tx_batch_delete,
            comm::db::message_add,
            comm::db::message_delete,
            comm::db::message_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}