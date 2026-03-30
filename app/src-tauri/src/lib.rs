use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipboardHistoryPayload {
    items: Vec<String>,
}

#[derive(Clone, Default)]
struct ClipboardState {
    items: Arc<Mutex<Vec<String>>>,
    selected_index: Arc<Mutex<usize>>,
}

#[tauri::command]
fn get_clipboard_history(state: tauri::State<'_, ClipboardState>) -> Vec<String> {
    if let Ok(items) = state.items.lock() {
        return items.clone();
    }

    Vec::new()
}

#[tauri::command]
fn set_selected_history_index(index: usize, state: tauri::State<'_, ClipboardState>) {
    if let Ok(mut selected_index) = state.selected_index.lock() {
        *selected_index = index;
    }
}

#[tauri::command]
fn bring_history_window_to_front(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_focusable(false);
        let _ = window.set_always_on_top(true);
        let _ = window.unminimize();
        let _ = window.show();
    }
}

#[tauri::command]
fn paste_selected_history_item(
    app: tauri::AppHandle,
    state: tauri::State<'_, ClipboardState>,
) -> Result<(), String> {
    let selected_index = state
        .selected_index
        .lock()
        .map_err(|_| "failed to lock selected index".to_string())
        .map(|selected| *selected)?;

    let selected_item = state
        .items
        .lock()
        .map_err(|_| "failed to lock clipboard history".to_string())
        .and_then(|items| {
            items
                .get(selected_index)
                .cloned()
                .ok_or_else(|| "no selected history item".to_string())
        })?;

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(selected_item)
        .map_err(|e| e.to_string())?;

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    thread::sleep(Duration::from_millis(30));

    // Simulate Ctrl+V so the previously focused app receives the selected history text.
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);

    Ok(())
}

fn push_history_item(state: &ClipboardState, value: String) -> Option<Vec<String>> {
    if value.trim().is_empty() {
        return None;
    }

    if let Ok(mut items) = state.items.lock() {
        if items.first().is_some_and(|current| current == &value) {
            return None;
        }

        items.retain(|item| item != &value);
        items.insert(0, value);

        const HISTORY_LIMIT: usize = 100;
        if items.len() > HISTORY_LIMIT {
            items.truncate(HISTORY_LIMIT);
        }

        return Some(items.clone());
    }

    None
}

fn start_clipboard_watcher(app: tauri::AppHandle, state: ClipboardState) {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                eprintln!("failed to initialize clipboard: {error}");
                return;
            }
        };

        let mut last_text = String::new();

        loop {
            if let Ok(text) = clipboard.get_text() {
                if text != last_text {
                    last_text = text.clone();
                    if let Some(items) = push_history_item(&state, text) {
                        let payload = ClipboardHistoryPayload { items };
                        let _ = app.emit("clipboard-history-updated", payload);
                    }
                }
            }

            thread::sleep(Duration::from_millis(400));
        }
    });
}

fn register_show_window_shortcut(app: &tauri::AppHandle) {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Digit7);
    let app_handle = app.clone();

    if let Err(error) = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
        bring_history_window_to_front(app_handle.clone());
    }) {
        eprintln!("failed to register global shortcut: {error}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let clipboard_state = ClipboardState::default();

    tauri::Builder::default()
        .manage(clipboard_state.clone())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            register_show_window_shortcut(app.handle());
            start_clipboard_watcher(app.handle().clone(), clipboard_state.clone());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focusable(false);
                let _ = window.set_always_on_top(true);
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            set_selected_history_index,
            paste_selected_history_item,
            bring_history_window_to_front
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
