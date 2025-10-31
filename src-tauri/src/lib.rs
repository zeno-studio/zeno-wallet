// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod db;
mod utils;
use tauri_plugin_sql::{Migration, MigrationKind};
use utils::password_check::{PasswordResult, check_password};

#[tauri::command]
fn check_password_cmd(pw: String) -> PasswordResult {
    check_password(&pw)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("sql/version1.sql"),
            kind: MigrationKind::Up,
        },
        // 示例：添加一个新的迁移来修改表结构
        // Migration {
        //     version: 2,
        //     description: "add_email_column_to_users_table",
        //     sql: "ALTER TABLE users ADD COLUMN email TEXT;",
        //     kind: MigrationKind::Up,
        // },
        // // 示例：添加另一个迁移来创建新表
        // Migration {
        //     version: 3,
        //     description: "create_transactions_table",
        //     sql: "CREATE TABLE transactions (id INTEGER PRIMARY KEY, user_id INTEGER, amount REAL, timestamp TEXT);",
        //     kind: MigrationKind::Up,
        // },
    ];
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:wallet.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_sql::Builder::new().build())
        .invoke_handler(tauri::generate_handler![check_password_cmd])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
