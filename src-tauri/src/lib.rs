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
mod apps;
mod helios;

use tauri::Manager;
use crate::helios::handler::helios_protocol_handler;

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
        .register_uri_scheme_protocol("helios", helios_protocol_handler)  

        .invoke_handler(tauri::generate_handler![
            // core 相关命令
            // Utils 相关命令
            utils::translate::set_lang,
            utils::translate::t,
            // Browser/DApp 相关命令
            dapp::dapp::get_darkmode,
            dapp::dapp::close_dapp_window,
            dapp::dapp::get_balance,
            dapp::dapp::sign_transaction,
            dapp::dapp::dapp_post_message,
            dapp::dapp::open_dapp_window,
            // Helios 相关命令
            // 可以在这里添加更多的 Helios 命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}