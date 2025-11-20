// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod dapp;
mod constants;
mod core;
mod error;
mod utils;
mod data;
mod rpc;
mod evm;
mod revm;


use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let appdb = core::db::AppDB::init(&app.handle())?;
            app.manage(appdb);

            app.manage(core::state::AppState::init(app.state())?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // core 相关命令
            // Utils 相关命令
            utils::i18n::set_lang,
            utils::i18n::t,
            // Browser/DApp 相关命令
            dapp::dapp::get_darkmode,
            dapp::dapp::close_dapp_window,
            dapp::dapp::get_balance,
            dapp::dapp::sign_transaction,
            dapp::dapp::dapp_post_message,
            dapp::dapp::open_dapp_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}