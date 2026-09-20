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
    Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
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
fn put_item(
    text: String,
    tags: Vec<String>,
    index: usize,
    state: tauri::State<'_, AppState>,
) -> Item {
    let item = Item {
        id: new_id(),
        text,
        tags,
    };
    state
        .store
        .lock()
        .expect("store")
        .insert_at(index, item.clone());
    item
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
fn paste_item(
    id: String,
    keep_open: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) {
    let text = state
        .store
        .lock()
        .expect("store")
        .get(&id)
        .map(|item| item.text.clone());
    if let Some(text) = text {
        paste_text(&app, &state, &text, keep_open);
    }
}

#[tauri::command]
fn copy_item(id: String, state: tauri::State<'_, AppState>) -> bool {
    let text = state
        .store
        .lock()
        .expect("store")
        .get(&id)
        .map(|item| item.text.clone());
    match text {
        Some(text) => clipboard::write_clipboard_text(&text),
        None => false,
    }
}

#[tauri::command]
fn nudge_window(dx: i32, dy: i32, app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let point = clamp_to_screen(
        &window,
        Point {
            x: pos.x + logical_to_physical(dx, scale),
            y: pos.y + logical_to_physical(dy, scale),
        },
    );
    *state.last_position.lock().expect("last_position") = Some(point);
    let _ = window.set_position(Position::Physical(PhysicalPosition::new(point.x, point.y)));
}

#[tauri::command]
fn resize_window(dw: i32, dh: i32, app: tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    // set_size takes the inner size, so measure the inner size too and keep the
    // frame (shadow and resize border) out of the calculation.
    let (Ok(inner), Ok(outer), Ok(pos)) = (
        window.inner_size(),
        window.outer_size(),
        window.outer_position(),
    ) else {
        return;
    };
    let frame_w = outer.width.saturating_sub(inner.width) as i32;
    let frame_h = outer.height.saturating_sub(inner.height) as i32;
    let scale = window.scale_factor().unwrap_or(1.0);
    let min_w = logical_to_physical(200, scale);
    let min_h = logical_to_physical(140, scale);
    let (mut max_w, mut max_h) = (i32::MAX, i32::MAX);
    if let Some(monitor) = window
        .monitor_from_point(pos.x as f64, pos.y as f64)
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())
    {
        let area = monitor.work_area();
        max_w = area.position.x + area.size.width as i32 - pos.x - frame_w;
        max_h = area.position.y + area.size.height as i32 - pos.y - frame_h;
    }
    let width = next_side(
        inner.width as i32,
        logical_to_physical(dw, scale),
        min_w,
        max_w,
    );
    let height = next_side(
        inner.height as i32,
        logical_to_physical(dh, scale),
        min_h,
        max_h,
    );
    let _ = window.set_size(Size::Physical(PhysicalSize::new(
        width as u32,
        height as u32,
    )));
}

fn paste_text(app: &tauri::AppHandle, state: &AppState, text: &str, keep_open: bool) {
    let _ = clipboard::write_clipboard_text(text);
    hide_window(app, state);
    let foreground = if keep_open {
        *state.foreground.lock().expect("foreground")
    } else {
        state.foreground.lock().expect("foreground").take()
    };
    let restored = match foreground {
        Some(foreground) => platform::restore_foreground(&foreground),
        None => true,
    };
    if restored {
        std::thread::sleep(Duration::from_millis(70));
        let _ = platform::simulate_paste();
    }
    if keep_open {
        reveal_picker(app, state);
    }
}

fn reveal_picker(app: &tauri::AppHandle, state: &AppState) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    let _ = window.set_always_on_top(true);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
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

fn logical_to_physical(value: i32, scale: f64) -> i32 {
    (value as f64 * scale).round() as i32
}

fn next_side(current: i32, delta: i32, min: i32, max: i32) -> i32 {
    if delta == 0 {
        return current;
    }
    (current + delta).max(min).min(max.max(min))
}

fn clamp_to_screen(window: &WebviewWindow, point: Point) -> Point {
    let Ok(size) = window.outer_size() else {
        return point;
    };
    clamp_point(window, point, size.width, size.height)
}

fn clamp_point(window: &WebviewWindow, point: Point, width: u32, height: u32) -> Point {
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
    let right = (left + area.size.width as i32 - width as i32).max(left);
    let bottom = (top + area.size.height as i32 - height as i32).max(top);
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
            put_item,
            update_item,
            hide_picker,
            paste_item,
            copy_item,
            nudge_window,
            resize_window,
            get_shortcuts,
            set_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_other_side_untouched() {
        assert_eq!(next_side(360, 24, 200, 1920), 384);
        assert_eq!(next_side(280, 0, 140, 1080), 280);
    }

    #[test]
    fn stops_at_the_minimum() {
        assert_eq!(next_side(210, -24, 200, 1920), 200);
        assert_eq!(next_side(200, -24, 200, 1920), 200);
    }

    #[test]
    fn stops_at_the_screen_edge() {
        assert_eq!(next_side(1000, 24, 200, 1010), 1010);
    }

    #[test]
    fn minimum_wins_over_a_smaller_maximum() {
        assert_eq!(next_side(200, 24, 200, 150), 200);
    }
}
