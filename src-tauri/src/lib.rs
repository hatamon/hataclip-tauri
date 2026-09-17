mod actions;
mod clipboard;
mod platform;
mod settings;
mod shortcuts;
mod store;
mod tray;

use platform::Point;
use settings::{Settings, Shortcuts};
use std::sync::Mutex;
use std::time::Duration;
use store::{new_id, Item, Store};
use tauri::{
    Emitter, Manager, PhysicalPosition, Position, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

pub(crate) struct AppState {
    store: Mutex<Store>,
    settings: Mutex<Settings>,
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
fn hide_picker(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    hide_window(&app, &state);
}

#[tauri::command]
fn get_shortcuts(state: tauri::State<'_, AppState>) -> Shortcuts {
    state.settings.lock().expect("settings").shortcuts().clone()
}

#[tauri::command]
fn set_shortcuts(
    register: String,
    show: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let next = Shortcuts { register, show };
    let previous = state.settings.lock().expect("settings").shortcuts().clone();
    if next == previous {
        return Ok(());
    }
    if let Err(error) = shortcuts::apply(&app, &next) {
        let _ = shortcuts::apply(&app, &previous);
        return Err(error);
    }
    state.settings.lock().expect("settings").set_shortcuts(next);
    Ok(())
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
    hide_window(app, state);
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

fn hide_window(app: &tauri::AppHandle, state: &AppState) {
    if let Some(window) = app.get_webview_window("main") {
        remember_position(&window, state);
        let _ = window.hide();
    }
}

fn remember_position(window: &WebviewWindow, state: &AppState) {
    if let Ok(position) = window.outer_position() {
        *state.last_position.lock().expect("last_position") = Some(Point {
            x: position.x,
            y: position.y,
        });
    }
}

pub(crate) fn open_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings".into()))
        .title("hataclip 設定")
        .inner_size(340.0, 260.0)
        .resizable(false)
        .build();
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

    let position = (*state.last_position.lock().expect("last_position"))
        .or_else(|| fallback_center(&window));
    if let Some(position) = position.map(|point| clamp_to_screen(&window, point)) {
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
    let size = window.outer_size().ok()?;
    let area = monitor.work_area();
    Some(Point {
        x: area.position.x + (area.size.width as i32 - size.width as i32) / 2,
        y: area.position.y + (area.size.height as i32 - size.height as i32) / 2,
    })
}

fn clamp_to_screen(window: &WebviewWindow, point: Point) -> Point {
    let Ok(size) = window.outer_size() else {
        return point;
    };
    let monitor = window
        .monitor_from_point(point.x as f64, point.y as f64)
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return point;
    };
    let area = monitor.work_area();
    let left = area.position.x;
    let top = area.position.y;
    let right = (left + area.size.width as i32 - size.width as i32).max(left);
    let bottom = (top + area.size.height as i32 - size.height as i32).max(top);
    Point {
        x: point.x.clamp(left, right),
        y: point.y.clamp(top, bottom),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("app data dir");
            let store = Store::load(dir.join("items.json"));
            let settings = Settings::load(dir.join("settings.json"));
            let shortcuts = settings.shortcuts().clone();
            app.manage(AppState {
                store: Mutex::new(store),
                settings: Mutex::new(settings),
                foreground: Mutex::new(None),
                last_position: Mutex::new(None),
            });
            tray::setup(app)?;
            if let Err(error) = shortcuts::apply(app.handle(), &shortcuts) {
                eprintln!("failed to register global shortcuts: {error}");
                open_settings(app.handle());
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
            paste_item,
            get_shortcuts,
            set_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
