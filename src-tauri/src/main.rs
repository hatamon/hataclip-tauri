// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::generate_handler;
use tauri::Manager;

#[tauri::command]
fn get_text_list() -> Vec<String> {
    vec![
        "テキスト1".to_string(),
        "テキスト2".to_string(),
        "テキスト3".to_string(),
    ]
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(generate_handler![get_text_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
