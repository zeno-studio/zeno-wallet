// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod core;
mod error;
mod constants;
mod browser;
mod utils;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            core::db::db_init(&app.handle()).unwrap();
            core::state::AppState::tauri_setup(&app.handle()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // core 相关命令
            core::db::vault_get,
            core::db::vault_add,
            core::db::addressbook_list,
            core::db::addressbook_add,
            core::db::addressbook_delete,
            core::db::tx_list,
            core::db::custom_rpc_list,
            core::db::custom_rpc_add,
            core::db::custom_rpc_delete,
            core::db::tx_add,
            core::db::tx_find,
            core::db::tx_delete,
            core::db::tx_batch_insert,
            core::db::tx_batch_delete,
            core::db::message_add,
            core::db::message_delete,
            core::db::message_list,
            // Utils 相关命令
            utils::ps_check::check_password_strength,
            utils::i18n::set_lang,
            utils::i18n::t,
            utils::qr::scan_qr,
            // Browser 相关命令
            browser::webview::open_dapp,
            browser::webview::wallet_request,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}