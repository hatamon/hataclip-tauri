mod actions;
mod clipboard;
mod editor;
mod platform;
mod settings;
mod shortcuts;
mod store;
mod text;
mod tray;

use chrono::Local;
use platform::Point;
use serde::Serialize;
use settings::{Settings, Shortcuts, WindowGeom};
use std::sync::Mutex;
use std::time::Duration;
use store::{Item, Store};
use tauri::{
    Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

pub(crate) struct AppState {
    store: Mutex<Store>,
    settings: Mutex<Settings>,
    foreground: Mutex<Option<platform::Foreground>>,
    last_position: Mutex<Option<Point>>,
}

/// 追加した行と、更新後の一覧。追加直後にその行を選ぶために両方返す。
#[derive(Serialize)]
struct Put {
    item: Item,
    items: Vec<Item>,
}

#[tauri::command]
fn list_items(state: tauri::State<'_, AppState>) -> Vec<Item> {
    view(&state)
}

#[tauri::command]
fn delete_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").delete_many(&ids);
    view(&state)
}

#[tauri::command]
fn undo_delete(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").undo_delete();
    view(&state)
}

#[tauri::command]
fn put_item(
    text: String,
    tags: Vec<String>,
    anchor_id: Option<String>,
    above: bool,
    state: tauri::State<'_, AppState>,
) -> Put {
    let item = Item::new(text, tags);
    state
        .store
        .lock()
        .expect("store")
        .insert_relative(anchor_id.as_deref(), above, item.clone());
    Put {
        items: view(&state),
        item,
    }
}

#[tauri::command]
fn update_item(
    id: String,
    text: String,
    tags: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    state.store.lock().expect("store").update(&id, text, tags);
    view(&state)
}

#[tauri::command]
fn set_tag(
    ids: Vec<String>,
    tag: String,
    add: bool,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    state.store.lock().expect("store").set_tag(&ids, &tag, add);
    view(&state)
}

#[tauri::command]
fn set_pinned(ids: Vec<String>, pinned: bool, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").set_pinned(&ids, pinned);
    view(&state)
}

/// 外部エディタを開く。終わるまで待つので、待ち時間は別スレッドに逃がす。
#[tauri::command]
fn edit_external(
    id: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some(item) = state.store.lock().expect("store").get(&id).cloned() else {
        return Err("編集する行がない".to_string());
    };
    let editor = editor::find().ok_or("nvim も $EDITOR も見つからない")?;
    let file = editor::write_temp_file(&item.id, &item.text).map_err(|error| error.to_string())?;
    hide_window(&app, &state);
    std::thread::spawn(move || {
        if let Some(text) = editor::run(&editor, &file) {
            let state = app.state::<AppState>();
            state
                .store
                .lock()
                .expect("store")
                .update(&id, text, item.tags);
        }
        let state = app.state::<AppState>();
        show_window(&app);
        let _ = app.emit("items-changed", view(&state));
    });
    Ok(())
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

/// 複数行まとめて貼るときは改行でつなぐ。format は Shift+Enter のときだけ真。
#[tauri::command]
fn paste_items(
    ids: Vec<String>,
    keep_open: bool,
    format: bool,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) {
    let Some(text) = joined_text(&state, &ids) else {
        return;
    };
    let text = if format {
        text::format_for_paste(&text)
    } else {
        text
    };
    let now = Local::now();
    let text = text::expand_template(
        &text,
        &now.format("%Y/%m/%d").to_string(),
        &now.format("%H:%M").to_string(),
    );
    if let Some(key) = current_context(&state) {
        state
            .store
            .lock()
            .expect("store")
            .record_context(&ids, &key);
    }
    paste_text(&app, &state, &text, keep_open);
}

#[tauri::command]
fn copy_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> bool {
    match joined_text(&state, &ids) {
        Some(text) => clipboard::write_clipboard_text(&text),
        None => false,
    }
}

fn joined_text(state: &AppState, ids: &[String]) -> Option<String> {
    let store = state.store.lock().expect("store");
    let texts: Vec<String> = ids
        .iter()
        .filter_map(|id| store.get(id).map(|item| item.text.clone()))
        .collect();
    if texts.is_empty() {
        None
    } else {
        Some(texts.join("\n"))
    }
}

/// 一覧に渡す並び。いまの貼り付け先で使った行を上に持ってくる。
fn view(state: &AppState) -> Vec<Item> {
    let context = current_context(state);
    let store = state.store.lock().expect("store");
    store::ordered(store.list(), context.as_deref())
}

fn current_context(state: &AppState) -> Option<String> {
    let foreground = *state.foreground.lock().expect("foreground");
    foreground.and_then(|foreground| platform::context_key(&foreground))
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
    let previous = clipboard::peek_text();
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
        if let Some(previous) = previous {
            std::thread::sleep(Duration::from_millis(200));
            let _ = clipboard::write_clipboard_text(&previous);
        }
    }
    if keep_open {
        reveal_picker(app, state);
    }
}

fn reveal_picker(app: &tauri::AppHandle, state: &AppState) {
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    show_window(app);
}

fn show_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.set_always_on_top(true);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_window(app: &tauri::AppHandle, state: &AppState) {
    if let Some(window) = app.get_webview_window("main") {
        persist_geometry(&window, state);
        let _ = window.hide();
    }
}

fn persist_geometry(window: &WebviewWindow, state: &AppState) {
    let Ok(position) = window.outer_position() else {
        return;
    };
    let Ok(inner) = window.inner_size() else {
        return;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let geom = WindowGeom {
        x: position.x,
        y: position.y,
        width: (inner.width as f64 / scale).round() as u32,
        height: (inner.height as f64 / scale).round() as u32,
    };
    *state.last_position.lock().expect("last_position") = Some(Point {
        x: position.x,
        y: position.y,
    });
    state.settings.lock().expect("settings").set_window(geom);
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

pub(crate) fn register_from_clipboard(app: &tauri::AppHandle, state: &AppState) {
    let previous = clipboard::peek_text();
    let _ = platform::simulate_copy();
    std::thread::sleep(Duration::from_millis(70));
    let captured = clipboard::peek_text();
    if let Some(previous) = previous {
        let _ = clipboard::write_clipboard_text(&previous);
    }
    let Some(text) = captured.filter(|text| !text.trim().is_empty()) else {
        return;
    };
    let tags = text::auto_tags(&text);
    state
        .store
        .lock()
        .expect("store")
        .insert(Item::new(text, tags));
    let _ = app.emit("items-changed", view(state));
}

pub(crate) fn show_picker(app: &tauri::AppHandle, state: &AppState) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    apply_saved_size(&window, state);

    let position = (*state.last_position.lock().expect("last_position"))
        .or_else(|| fallback_center(&window));
    if let Some(position) = position.map(|point| clamp_to_screen(&window, point)) {
        *state.last_position.lock().expect("last_position") = Some(position);
        let _ = window.set_position(Position::Physical(PhysicalPosition::new(
            position.x,
            position.y,
        )));
    }

    show_window(app);
    let _ = app.emit("picker-opened", view(state));
}

fn apply_saved_size(window: &WebviewWindow, state: &AppState) {
    let Some(geom) = state.settings.lock().expect("settings").window() else {
        return;
    };
    if geom.width < 200 || geom.height < 140 {
        return;
    }
    let _ = window.set_size(Size::Logical(LogicalSize::new(
        f64::from(geom.width),
        f64::from(geom.height),
    )));
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
            let last_position = Mutex::new(settings.window().map(|geom| Point {
                x: geom.x,
                y: geom.y,
            }));
            app.manage(AppState {
                store: Mutex::new(store),
                settings: Mutex::new(settings),
                foreground: Mutex::new(None),
                last_position,
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
            delete_items,
            undo_delete,
            put_item,
            update_item,
            set_tag,
            set_pinned,
            edit_external,
            hide_picker,
            paste_items,
            copy_items,
            nudge_window,
            resize_window,
            get_shortcuts,
            set_shortcuts
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
            ) {
                if let Some(window) = app.get_webview_window("main") {
                    persist_geometry(&window, &app.state::<AppState>());
                }
            }
        });
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
