use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClipboardHistoryPayload {
    items: Vec<String>,
}

#[derive(Clone, Default)]
struct ClipboardState {
    items: Arc<Mutex<Vec<String>>>,
    selected_index: Arc<Mutex<usize>>,
    previous_foreground_window: Arc<Mutex<isize>>,
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
fn bring_history_window_to_front(app: tauri::AppHandle, state: tauri::State<'_, ClipboardState>) {
    bring_history_window_to_front_impl(&app, &state);
}

#[tauri::command]
fn hide_history_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn bring_history_window_to_front_impl(app: &tauri::AppHandle, state: &ClipboardState) {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
            if let (Ok(hwnd), Some(foreground)) = (window.hwnd(), get_foreground_window_handle()) {
                let current_window = hwnd.0 as isize;
                if foreground != current_window {
                    let _ = state.previous_foreground_window.lock().map(|mut saved| {
                        *saved = foreground;
                    });
                }
            }
        }

        let _ = window.set_focusable(true);
        let _ = window.set_always_on_top(true);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn paste_selected_history_item(
    app: tauri::AppHandle,
    state: tauri::State<'_, ClipboardState>,
) -> Result<(), String> {
    paste_selected_history_item_impl(&app, &state)
}

fn paste_selected_history_item_impl(
    app: &tauri::AppHandle,
    state: &ClipboardState,
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
    restore_previous_foreground_window(state);
    thread::sleep(Duration::from_millis(60));

    // Simulate Ctrl+V so the previously focused app receives the selected history text.
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::Control, Press);
        let _ = enigo.key(Key::Unicode('v'), Click);
        let _ = enigo.key(Key::Control, Release);
    } else {
        return Err("failed to initialize keyboard input backend".to_string());
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn get_foreground_window_handle() -> Option<isize> {
    // SAFETY: Calling Win32 API to read current foreground window handle.
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return None;
    }
    Some(hwnd as isize)
}

#[cfg(not(target_os = "windows"))]
fn get_foreground_window_handle() -> Option<isize> {
    None
}

fn restore_previous_foreground_window(state: &ClipboardState) {
    #[cfg(target_os = "windows")]
    {
        let saved = state
            .previous_foreground_window
            .lock()
            .ok()
            .map(|value| *value)
            .unwrap_or(0);
        if saved != 0 {
            // SAFETY: Uses a handle previously observed from GetForegroundWindow.
            unsafe {
                let _ = SetForegroundWindow(saved as _);
            }
        }
    }
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

fn register_show_window_shortcut(app: &tauri::AppHandle, state: ClipboardState) {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Digit7);
    let app_handle = app.clone();

    if let Err(error) = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
        bring_history_window_to_front_impl(&app_handle, &state);
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
            register_show_window_shortcut(app.handle(), clipboard_state.clone());
            start_clipboard_watcher(app.handle().clone(), clipboard_state.clone());

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focusable(true);
                let _ = window.set_always_on_top(true);
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            set_selected_history_index,
            paste_selected_history_item,
            bring_history_window_to_front,
            hide_history_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
