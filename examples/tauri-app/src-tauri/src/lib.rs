#[macro_use]
extern crate dotenv_codegen;

use tauri_plugin_authium::AuthiumConfig;

// Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv::dotenv().ok();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet
        ])
        .plugin(tauri_plugin_authium::init(Some(AuthiumConfig::new(
            dotenv!("API_KEY").into(),
            dotenv!("APP_ID").into(),
        ))))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
