mod actions;
mod clipboard;
mod platform;
mod shortcuts;
mod store;

use platform::Point;
use std::sync::Mutex;
use std::time::Duration;
use store::{new_id, Item, Store};
use tauri::{Emitter, Manager, PhysicalPosition, Position, WebviewWindow};

pub(crate) struct AppState {
    store: Mutex<Store>,
    foreground: Mutex<Option<platform::Foreground>>,
    last_position: Mutex<Option<Point>>,
}

#[tauri::command]
fn list_items(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").list().to_vec()
}

#[tauri::command]
fn delete_item(id: String, state: tauri::State<'_, AppState>) -> bool {
    state.store.lock().expect("store").delete(&id)
}

#[tauri::command]
fn update_item(
    id: String,
    text: String,
    tags: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> bool {
    state.store.lock().expect("store").update(&id, text, tags)
}

#[tauri::command]
fn hide_picker(app: tauri::AppHandle) {
    hide_window(&app);
}

#[tauri::command]
fn paste_item(id: String, app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    let text = state
        .store
        .lock()
        .expect("store")
        .get(&id)
        .map(|item| item.text.clone());
    if let Some(text) = text {
        paste_text(&app, &state, &text);
    }
}

fn paste_text(app: &tauri::AppHandle, state: &AppState, text: &str) {
    let _ = clipboard::write_clipboard_text(text);
    hide_window(app);
    let foreground = state.foreground.lock().expect("foreground").take();
    let restored = match foreground {
        Some(foreground) => platform::restore_foreground(&foreground),
        None => true,
    };
    if !restored {
        return;
    }
    std::thread::sleep(Duration::from_millis(70));
    let _ = platform::simulate_paste();
}

fn hide_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn insert_item(state: &AppState, text: String) {
    let item = Item {
        id: new_id(),
        text,
        tags: Vec::new(),
    };
    state.store.lock().expect("store").insert(item);
}

pub(crate) fn register_from_clipboard(app: &tauri::AppHandle, state: &AppState) {
    let Some(text) = clipboard::read_clipboard_text() else {
        return;
    };
    insert_item(state, text);
    let items = state.store.lock().expect("store").list().to_vec();
    let _ = app.emit("items-changed", items);
}

pub(crate) fn show_picker(app: &tauri::AppHandle, state: &AppState) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();

    let position = platform::anchor_position()
        .or_else(|| *state.last_position.lock().expect("last_position"))
        .or_else(|| fallback_center(&window));
    if let Some(position) = position {
        *state.last_position.lock().expect("last_position") = Some(position);
        let _ = window.set_position(Position::Physical(PhysicalPosition::new(
            position.x,
            position.y,
        )));
    }

    let _ = window.set_always_on_top(true);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();

    let items = state.store.lock().expect("store").list().to_vec();
    let _ = app.emit("picker-opened", items);
}

fn fallback_center(window: &WebviewWindow) -> Option<Point> {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())?;
    let pos = monitor.position();
    let size = monitor.size();
    Some(Point {
        x: pos.x + (size.width as i32 / 2) - 180,
        y: pos.y + (size.height as i32 / 2) + 40,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            let store = Store::load(dir.join("items.json"));
            app.manage(AppState {
                store: Mutex::new(store),
                foreground: Mutex::new(None),
                last_position: Mutex::new(None),
            });
            if let Err(error) = shortcuts::register(app.handle()) {
                eprintln!("failed to register global shortcuts: {error}");
                let state = app.state::<AppState>();
                show_picker(app.handle(), &state);
            } else if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_items,
            delete_item,
            update_item,
            hide_picker,
            paste_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
