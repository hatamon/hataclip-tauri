mod actions;
mod chord;
mod clipboard;
mod keys;
mod editor;
mod eval;
mod expr;
mod help;
mod platform;
mod settings;
mod shell;
mod shortcuts;
mod store;
mod text;
mod tray;

use chrono::Local;
use platform::Point;
use serde::Serialize;
use settings::{KeyMaps, NCommand, SetCommand, Settings, Shortcuts, WindowGeom};
use std::collections::HashMap;
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
    paste_serial: Mutex<u32>,
    last_sh: Mutex<Option<String>>,
    picker_open: Mutex<bool>,
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
fn undo_change(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").undo();
    view(&state)
}

#[tauri::command]
fn redo_change(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").redo();
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
    view_keeping(&state, &[id])
}

#[tauri::command]
fn set_tag(
    ids: Vec<String>,
    tag: String,
    add: bool,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    state.store.lock().expect("store").set_tag(&ids, &tag, add);
    view_keeping(&state, &ids)
}

#[tauri::command]
fn set_app_tag(
    ids: Vec<String>,
    app: String,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    state.store.lock().expect("store").set_app_tag(&ids, &app);
    view_keeping(&state, &ids)
}

#[tauri::command]
fn set_pinned(ids: Vec<String>, pinned: bool, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").set_pinned(&ids, pinned);
    view(&state)
}

#[tauri::command]
fn move_pins(ids: Vec<String>, delta: i32, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").move_pins(&ids, delta);
    view(&state)
}

#[tauri::command]
fn split_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").split_items(&ids);
    view(&state)
}

#[tauri::command]
fn get_context(state: tauri::State<'_, AppState>) -> Option<String> {
    current_context(&state)
}

#[tauri::command]
fn merge_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").merge_items(&ids);
    view(&state)
}

#[tauri::command]
fn clone_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").clone_items(&ids);
    view(&state)
}

#[tauri::command]
fn substitute_items(
    ids: Vec<String>,
    old: String,
    new: String,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    state
        .store
        .lock()
        .expect("store")
        .substitute(&ids, &old, &new);
    view(&state)
}

#[tauri::command]
fn dedup_items(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").dedup();
    view(&state)
}

#[tauri::command]
fn sort_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").sort_items(&ids);
    view(&state)
}

#[tauri::command]
fn drop_paths(paths: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").drop_paths(&paths);
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
    let editor = editor::find().ok_or("エディタが見つからない")?;
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
    reset_n(&state);
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
    quick_paste: bool,
    expand: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let next = Shortcuts {
        register,
        show,
        quick_paste,
        expand,
    };
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
fn pause_shortcuts(app: tauri::AppHandle) {
    shortcuts::pause(&app);
}

#[tauri::command]
fn resume_shortcuts(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let shortcuts = state.settings.lock().expect("settings").shortcuts().clone();
    shortcuts::resume(&app, &shortcuts)
}

#[tauri::command]
fn get_keymaps(state: tauri::State<'_, AppState>) -> KeyMaps {
    state.settings.lock().expect("settings").keymaps().clone()
}

#[tauri::command]
fn set_keymaps(keymaps: KeyMaps, state: tauri::State<'_, AppState>) -> KeyMaps {
    let mut settings = state.settings.lock().expect("settings");
    settings.set_keymaps(keymaps);
    settings.keymaps().clone()
}

#[tauri::command]
fn open_settings_file(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let editor = editor::find().ok_or("エディタが見つからない")?;
    let path = {
        let settings = state.settings.lock().expect("settings");
        settings.persist();
        settings.path().to_path_buf()
    };
    hide_window(&app, &state);
    std::thread::spawn(move || {
        let _ = editor::wait_close(&editor, &path);
        let state = app.state::<AppState>();
        let shortcuts = {
            let mut settings = state.settings.lock().expect("settings");
            let _ = settings.reload();
            settings.shortcuts().clone()
        };
        if let Err(error) = shortcuts::apply(&app, &shortcuts) {
            eprintln!("failed to register global shortcuts: {error}");
        }
        let keymaps = state.settings.lock().expect("settings").keymaps().clone();
        let _ = app.emit("settings-changed", keymaps);
        show_window(&app);
    });
    Ok(())
}

#[tauri::command]
fn apply_set(rest: String, state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    match settings::parse_set(&rest) {
        Some(SetCommand::List) => Ok(Some(
            state.settings.lock().expect("settings").format_vars(),
        )),
        Some(SetCommand::Paste(spec)) => {
            let app = foreground_app(&state);
            let _ = state
                .settings
                .lock()
                .expect("settings")
                .set_target_paste(&app, &spec);
            Ok(None)
        }
        Some(SetCommand::Var { name, value }) => {
            if !state
                .settings
                .lock()
                .expect("settings")
                .set_var(&name, value)
            {
                return Err("変数名が使えない".into());
            }
            Ok(None)
        }
        None => Err("書き方が違う".into()),
    }
}

#[tauri::command]
fn apply_n(rest: String, state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    match settings::parse_n(&rest) {
        Some(NCommand::Show) => Ok(Some(format_n(&state))),
        Some(NCommand::Set(n)) => {
            state.settings.lock().expect("settings").set_n(n);
            *state.paste_serial.lock().expect("paste_serial") = n;
            Ok(None)
        }
        None => Err("書き方が違う".into()),
    }
}

/// 複数行まとめて貼るときは separator でつなぐ。format は Shift+Enter のときだけ真。
#[tauri::command]
fn paste_items(
    ids: Vec<String>,
    keep_open: bool,
    format: bool,
    raw: Option<bool>,
    separator: Option<String>,
    answers: Option<HashMap<String, String>>,
    prefix: Option<String>,
    typed: Option<bool>,
    resolved: Option<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> bool {
    if let Some(text) = resolved {
        return paste_resolved_text(
            &app,
            &state,
            &ids,
            &text,
            keep_open,
            format,
            prefix.as_deref(),
        );
    }
    run_paste(
        &app,
        &state,
        &ids,
        keep_open,
        format,
        raw.unwrap_or(false),
        separator.as_deref().unwrap_or("\n"),
        answers.unwrap_or_default(),
        prefix.as_deref(),
        typed.unwrap_or(false),
        false,
    )
}

fn paste_resolved_text(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    text: &str,
    keep_open: bool,
    format: bool,
    prefix: Option<&str>,
) -> bool {
    let mut text = if format {
        text::format_for_paste(text)
    } else {
        text.to_string()
    };
    if let Some(prefix) = prefix {
        let already = prefix.chars().next().map(|ch| ch.to_string()).unwrap_or_default();
        let already = if prefix == "* " { "* " } else { already.as_str() };
        text = text::prefix_lines(&text, prefix, already);
    }
    if let Some(key) = current_context(state) {
        state.store.lock().expect("store").record_context(ids, &key);
    }
    state.store.lock().expect("store").bump_paste(ids);
    paste_text(app, state, &text, keep_open);
    drop_once(state, ids);
    let _ = app.emit("items-changed", view(state));
    bump_n(state);
    if !keep_open {
        reset_n(state);
    }
    true
}

#[tauri::command]
fn copy_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> bool {
    match expand_ids(&state, &ids, true, HashMap::new()) {
        Some(text) => clipboard::write_clipboard_text(&text),
        None => false,
    }
}

#[tauri::command]
fn expand_items(
    ids: Vec<String>,
    answers: Option<HashMap<String, String>>,
    force_sh: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Option<String> {
    expand_ids(
        &state,
        &ids,
        force_sh.unwrap_or(false),
        answers.unwrap_or_default(),
    )
}

#[tauri::command]
fn expand_text(text: String, state: tauri::State<'_, AppState>) -> String {
    let ctx = expand_context(&state, String::new(), HashMap::new());
    text::expand_template(&text, &ctx)
}

#[tauri::command]
fn list_tags(state: tauri::State<'_, AppState>) -> String {
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for item in view(&state) {
        for tag in &item.tags {
            if tag.is_empty() {
                continue;
            }
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    if counts.is_empty() {
        return "タグはない".to_string();
    }
    counts
        .iter()
        .map(|(name, count)| format!("#{name}  {count}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn expand_ids(
    state: &AppState,
    ids: &[String],
    force_sh: bool,
    answers: HashMap<String, String>,
) -> Option<String> {
    let ctx = expand_context(state, String::new(), answers);
    let store = state.store.lock().expect("store");
    let rows: Vec<Item> = ids
        .iter()
        .filter_map(|id| store.get(id).cloned())
        .collect();
    drop(store);
    if rows.len() != ids.len() {
        return None;
    }
    let mut parts = Vec::new();
    for item in &rows {
        let ops = resolve_item(item, &ctx, force_sh)?;
        parts.push(text::flatten_ops(&ops));
    }
    Some(parts.join("\n"))
}

#[tauri::command]
fn open_target(text: String) -> bool {
    open_target_text(&text)
}

#[tauri::command]
fn get_help(topic: Option<String>) -> String {
    help::render(topic.as_deref())
}

#[tauri::command]
fn help_topics() -> Vec<String> {
    help::topics()
}

#[tauri::command]
fn export_items(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("パスが空".into());
    }
    let store = state.store.lock().expect("store");
    let mut md = String::from("# hataclip\n\n");
    for item in store.list() {
        md.push_str("## ");
        let title = item.text.lines().next().unwrap_or("").trim();
        md.push_str(if title.is_empty() { "(empty)" } else { title });
        md.push('\n');
        if !item.tags.is_empty() {
            md.push_str("tags: ");
            md.push_str(&item.tags.join(", "));
            md.push('\n');
        }
        if item.pinned {
            md.push_str("pinned: true\n");
        }
        md.push_str("\n```\n");
        md.push_str(&item.text);
        md.push_str("\n```\n\n");
    }
    std::fs::write(path, md).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_items(path: String, state: tauri::State<'_, AppState>) -> Result<Vec<Item>, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(view(&state));
    }
    let markdown = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(_) => return Ok(view(&state)),
    };
    state
        .store
        .lock()
        .expect("store")
        .import_markdown(&markdown);
    Ok(view(&state))
}

#[tauri::command]
fn clear_unpinned(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").clear_unpinned();
    view(&state)
}

#[tauri::command]
fn paste_script(
    script: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let script = script.trim().to_string();
    if script.is_empty() {
        return Ok(());
    }
    let output = shell::run_script(&script).map_err(|_| "コマンドに失敗した".to_string())?;
    *state.last_sh.lock().expect("last_sh") = Some(script);
    paste_text(&app, &state, &output, false);
    Ok(())
}

#[tauri::command]
fn paste_echo(
    expr: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Ok(());
    }
    let vars = var_map(&state);
    let value = expr::eval_with(expr, &vars).ok_or("計算できない")?;
    paste_text(&app, &state, &expr::format_number(value), false);
    Ok(())
}

#[tauri::command]
fn paste_last_script(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some(script) = state.last_sh.lock().expect("last_sh").clone() else {
        return Ok(());
    };
    paste_script(script, app, state)
}

pub(crate) fn paste_ranked(index: usize, app: &tauri::AppHandle, state: &AppState) {
    if *state.picker_open.lock().expect("picker_open") {
        return;
    }
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    let items = view(state);
    let Some(item) = crate::text::ranked_index(&items, index, |item| item.tags.as_slice())
        .and_then(|i| items.get(i))
    else {
        return;
    };
    run_paste(
        app,
        state,
        &[item.id.clone()],
        false,
        false,
        false,
        "\n",
        HashMap::new(),
        None,
        false,
        false,
    );
}

pub(crate) fn paste_from_selection(app: &tauri::AppHandle, state: &AppState) {
    if *state.picker_open.lock().expect("picker_open") {
        return;
    }
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    let previous = clipboard::peek_text();
    let spec = copy_spec(state);
    let _ = platform::simulate_copy(&spec);
    std::thread::sleep(Duration::from_millis(70));
    let captured = clipboard::peek_text();
    if let Some(previous) = previous.as_ref() {
        let _ = clipboard::write_clipboard_text(previous);
    }
    let Some(text) = captured else {
        return;
    };
    let ctx = expand_context(state, String::new(), HashMap::new());
    let last_sh = state.last_sh.lock().expect("last_sh").clone();
    let Some(out) = eval::resolve_selection(&text, &ctx, last_sh.as_deref()) else {
        return;
    };
    remember_last_sh(&text, state);
    play_resolved(app, state, text::take_type_ops(&out), false, false);
}

fn remember_last_sh(text: &str, state: &AppState) {
    let body = text.trim().strip_prefix(':').unwrap_or(text.trim()).trim();
    if let Some(script) = body.strip_prefix("sh ") {
        let script = script.trim();
        if !script.is_empty() {
            *state.last_sh.lock().expect("last_sh") = Some(script.to_string());
        }
    }
}

fn open_target_text(text: &str) -> bool {
    let text = text.trim();
    if text.starts_with("http://") || text.starts_with("https://") {
        return open_url(text);
    }
    if !text::looks_like_path(text) {
        return false;
    }
    let path = std::path::Path::new(text);
    if path.is_file() {
        return std::process::Command::new("explorer")
            .arg(format!("/select,{text}"))
            .spawn()
            .is_ok();
    }
    if path.is_dir() {
        return std::process::Command::new("explorer")
            .arg(text)
            .spawn()
            .is_ok();
    }
    false
}

fn open_url(url: &str) -> bool {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .is_ok()
}

fn run_paste(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    keep_open: bool,
    format: bool,
    raw: bool,
    separator: &str,
    answers: HashMap<String, String>,
    prefix: Option<&str>,
    typed: bool,
    quiet: bool,
) -> bool {
    let was_open = *state.picker_open.lock().expect("picker_open");
    let needs_sel = !raw && !quiet && has_sel(state, ids);
    let sel = if needs_sel {
        capture_selection(app, state)
    } else {
        String::new()
    };
    let ctx = if raw {
        None
    } else {
        Some(expand_context(state, sel, answers))
    };
    let store = state.store.lock().expect("store");
    let rows: Vec<Item> = ids
        .iter()
        .filter_map(|id| store.get(id).cloned())
        .collect();
    drop(store);
    if rows.len() != ids.len() {
        if was_open {
            show_window(app);
        }
        return false;
    }
    let mut paste_parts: Vec<Vec<text::PasteOp>> = Vec::new();
    let mut logged = false;
    for item in &rows {
        let resolved = if let Some(ctx) = ctx.as_ref() {
                    match resolve_item(item, ctx, false) {
                Some(ops) => ops,
                None => {
                    if was_open {
                        show_window(app);
                    }
                    return false;
                }
            }
        } else {
            vec![text::PasteOp::Text(item.text.clone())]
        };
        if let Some(path) = text::log_path(&item.tags) {
            if append_log(&path, &text::flatten_ops(&resolved)).is_err() {
                if was_open {
                    show_window(app);
                }
                return false;
            }
            logged = true;
        } else {
            paste_parts.push(resolved);
        }
    }
    let mut ops = Vec::new();
    for (index, part) in paste_parts.iter().enumerate() {
        if index > 0 {
            ops.push(text::PasteOp::Text(separator.to_string()));
        }
        ops.extend(part.clone());
    }
    let mut ops = text::compact_ops(ops);
    if format {
        ops = map_text_ops(ops, |text| text::format_for_paste(text));
    }
    if let Some(prefix) = prefix {
        let already = prefix.chars().next().map(|ch| ch.to_string()).unwrap_or_default();
        let already = if prefix == "* " { "* " } else { already.as_str() };
        let already = already.to_string();
        ops = map_text_ops(ops, |text| text::prefix_lines(text, prefix, &already));
    }
    if paste_parts.is_empty() {
        if !logged {
            if was_open {
                show_window(app);
            }
            return false;
        }
        if let Some(key) = current_context(state) {
            state.store.lock().expect("store").record_context(ids, &key);
        }
        state.store.lock().expect("store").bump_paste(ids);
        drop_once(state, ids);
        hide_window(app, state);
        let _ = app.emit("items-changed", view(state));
        if keep_open {
            reveal_picker(app, state);
        }
        bump_n(state);
        if !keep_open {
            reset_n(state);
        }
        return true;
    }
    if text::flatten_ops(&ops).is_empty() && prefix.is_some() && !text::has_keys(&ops) {
        if was_open {
            show_window(app);
        }
        return false;
    }
    if let Some(key) = current_context(state) {
        state
            .store
            .lock()
            .expect("store")
            .record_context(ids, &key);
    }
    state.store.lock().expect("store").bump_paste(ids);
    let ok = play_resolved(app, state, ops, keep_open, typed);
    if ok {
        drop_once(state, ids);
        let _ = app.emit("items-changed", view(state));
        bump_n(state);
    } else if was_open {
        reveal_picker(app, state);
    }
    if !keep_open {
        reset_n(state);
    }
    ok
}

/// いまの一覧の並びで、先に出てきた `#alias:name` の本文。
fn alias_map(state: &AppState) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for item in view(state) {
        for tag in &item.tags {
            if let Some(name) = tag.strip_prefix("alias:") {
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                map.entry(name.to_string())
                    .or_insert_with(|| item.text.clone());
            }
        }
    }
    map
}

/// いまの一覧の並びで、同じタグの本文を改行つなぎ。
fn tag_map(state: &AppState) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for item in view(state) {
        for tag in &item.tags {
            if tag.is_empty() {
                continue;
            }
            map.entry(tag.clone())
                .and_modify(|body| {
                    body.push('\n');
                    body.push_str(&item.text);
                })
                .or_insert_with(|| item.text.clone());
        }
    }
    map
}

fn var_map(state: &AppState) -> HashMap<String, String> {
    state
        .settings
        .lock()
        .expect("settings")
        .vars()
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

fn peek_n(state: &AppState) -> u32 {
    *state.paste_serial.lock().expect("paste_serial")
}

fn bump_n(state: &AppState) {
    let mut serial = state.paste_serial.lock().expect("paste_serial");
    *serial = serial.saturating_add(1);
}

fn reset_n(state: &AppState) {
    let n = state.settings.lock().expect("settings").n();
    *state.paste_serial.lock().expect("paste_serial") = n;
}

fn format_n(state: &AppState) -> String {
    let start = state.settings.lock().expect("settings").n();
    let next = peek_n(state);
    if start == next {
        format!("{start}")
    } else {
        format!("初期値 {start}\n次 {next}")
    }
}

fn expand_context(
    state: &AppState,
    sel: String,
    answers: HashMap<String, String>,
) -> text::Expand {
    let (app_name, front) = {
        let foreground = *state.foreground.lock().expect("foreground");
        foreground
            .map(|foreground| platform::app_and_title(&foreground))
            .unwrap_or_default()
    };
    let n = peek_n(state);
    let now = Local::now();
    text::Expand {
        date: now.format("%Y/%m/%d").to_string(),
        time: now.format("%H:%M").to_string(),
        clip: clipboard::peek_text().unwrap_or_default(),
        sel,
        n,
        uuid: uuid::Uuid::new_v4().to_string(),
        user: text::login_name(),
        host: text::host_name(),
        app: app_name,
        front,
        now,
        answers,
        aliases: alias_map(state),
        vars: var_map(state),
        tags: tag_map(state),
    }
}

fn has_sel(state: &AppState, ids: &[String]) -> bool {
    let store = state.store.lock().expect("store");
    ids.iter().any(|id| {
        store
            .get(id)
            .map_or(false, |item| text::has_sel_token(&item.text))
    })
}

fn drop_once(state: &AppState, ids: &[String]) {
    let mut store = state.store.lock().expect("store");
    let once: Vec<String> = ids
        .iter()
        .filter(|id| {
            store
                .get(id)
                .map_or(false, |item| {
                    item.tags.iter().any(|tag| tag == "once") && !item.locked()
                })
        })
        .cloned()
        .collect();
    if !once.is_empty() {
        store.remove_ids(&once);
    }
}

fn capture_selection(app: &tauri::AppHandle, state: &AppState) -> String {
    let previous = clipboard::peek_text();
    let wait = *state.picker_open.lock().expect("picker_open");
    if wait {
        hide_window(app, state);
        let foreground = *state.foreground.lock().expect("foreground");
        if let Some(foreground) = foreground.as_ref() {
            let _ = platform::restore_foreground(foreground);
        }
        std::thread::sleep(Duration::from_millis(70));
    }
    let _ = platform::simulate_copy(&copy_spec(state));
    std::thread::sleep(Duration::from_millis(70));
    let captured = clipboard::peek_text();
    if let Some(previous) = previous.as_ref() {
        let _ = clipboard::write_clipboard_text(previous);
    }
    match captured {
        Some(text) if previous.as_ref() != Some(&text) => text,
        _ => String::new(),
    }
}

fn resolve_item(item: &Item, ctx: &text::Expand, force_sh: bool) -> Option<Vec<text::PasteOp>> {
    let run = item.tags.iter().any(|tag| tag == "run");
    let file = item.tags.iter().any(|tag| tag == "file");
    if run && !shell::has_sh_token(&item.text) {
        let expanded = text::expand_template(&item.text, ctx);
        return Some(vec![text::PasteOp::Text(apply_tsv(
            item,
            shell::run_script(&expanded).ok()?,
        )?)]);
    }
    let expanded = text::expand_template(&item.text, ctx);
    let expanded = shell::apply_sh(&expanded, run || force_sh).ok()?;
    let expanded = if file && !run {
        shell::read_file_contents(&text::flatten_ops(&text::take_type_ops(&expanded))).ok()?
    } else {
        expanded
    };
    let ops = text::take_type_ops(&expanded);
    if text::has_keys(&ops) {
        Some(ops)
    } else {
        Some(vec![text::PasteOp::Text(apply_tsv(
            item,
            text::flatten_ops(&ops),
        )?)])
    }
}

fn map_text_ops(ops: Vec<text::PasteOp>, map: impl Fn(&str) -> String) -> Vec<text::PasteOp> {
    text::compact_ops(
        ops.into_iter()
            .map(|op| match op {
                text::PasteOp::Text(text) => text::PasteOp::Text(map(&text)),
                other => other,
            })
            .collect(),
    )
}

fn play_resolved(
    app: &tauri::AppHandle,
    state: &AppState,
    ops: Vec<text::PasteOp>,
    keep_open: bool,
    typed: bool,
) -> bool {
    if !text::has_keys(&ops) {
        let text = text::flatten_ops(&ops);
        if typed {
            return type_text(app, state, &text, keep_open);
        }
        paste_text(app, state, &text, keep_open);
        return true;
    }
    play_ops(app, state, &ops, keep_open, typed)
}

fn play_ops(
    app: &tauri::AppHandle,
    state: &AppState,
    ops: &[text::PasteOp],
    keep_open: bool,
    typed: bool,
) -> bool {
    let previous = clipboard::peek_text();
    let spec = paste_spec(state);
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        return false;
    }
    if wait {
        std::thread::sleep(Duration::from_millis(70));
    }
    let mut did_paste = false;
    let restore = |did_paste: bool| {
        if did_paste {
            if let Some(previous) = previous.as_ref() {
                std::thread::sleep(Duration::from_millis(200));
                let _ = clipboard::write_clipboard_text(previous);
            }
        }
    };
    for (index, op) in ops.iter().enumerate() {
        if index > 0 {
            std::thread::sleep(Duration::from_millis(70));
        }
        let ok = match op {
            text::PasteOp::Text(text) if text.is_empty() => true,
            text::PasteOp::Text(text) if typed => platform::simulate_type(text),
            text::PasteOp::Text(text) => {
                let _ = clipboard::write_clipboard_text(text);
                did_paste = true;
                platform::simulate_paste(&spec)
            }
            text::PasteOp::Type(atoms) => platform::simulate_type_atoms(atoms),
            text::PasteOp::Wait(ms) => {
                if *ms > 0 {
                    std::thread::sleep(Duration::from_millis(*ms));
                }
                true
            }
        };
        if !ok {
            if did_paste {
                if let Some(previous) = previous.as_ref() {
                    let _ = clipboard::write_clipboard_text(previous);
                }
            }
            return false;
        }
    }
    restore(did_paste);
    if keep_open {
        reveal_picker(app, state);
    }
    true
}

fn apply_tsv(item: &Item, text: String) -> Option<String> {
    if item.tags.iter().any(|tag| tag == "tsv") {
        text::to_tsv(&text)
    } else {
        Some(text)
    }
}

fn append_log(path: &str, text: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    file.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// 一覧に渡す並び。ピンの下はストアの順。
fn view(state: &AppState) -> Vec<Item> {
    view_keeping(state, &[])
}

fn view_keeping(state: &AppState, keep: &[String]) -> Vec<Item> {
    let context = current_context(state);
    let store = state.store.lock().expect("store");
    store::ordered_keeping(store.list(), context.as_deref(), keep)
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

fn foreground_app(state: &AppState) -> String {
    (*state.foreground.lock().expect("foreground"))
        .map(|fg| platform::app_and_title(&fg).0)
        .unwrap_or_default()
}

fn copy_spec(state: &AppState) -> String {
    let app = foreground_app(state);
    state.settings.lock().expect("settings").copy_for(&app)
}

fn paste_spec(state: &AppState) -> String {
    let app = foreground_app(state);
    state.settings.lock().expect("settings").paste_for(&app)
}

fn yield_target(app: &tauri::AppHandle, state: &AppState, keep_open: bool) -> bool {
    if !*state.picker_open.lock().expect("picker_open") {
        return true;
    }
    hide_window(app, state);
    let foreground = if keep_open {
        *state.foreground.lock().expect("foreground")
    } else {
        state.foreground.lock().expect("foreground").take()
    };
    match foreground {
        Some(foreground) => platform::restore_foreground(&foreground),
        None => true,
    }
}

fn paste_text(app: &tauri::AppHandle, state: &AppState, text: &str, keep_open: bool) {
    let previous = clipboard::peek_text();
    let _ = clipboard::write_clipboard_text(text);
    let spec = paste_spec(state);
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        if keep_open {
            reveal_picker(app, state);
        }
        return;
    }
    if wait {
        std::thread::sleep(Duration::from_millis(70));
    }
    let _ = platform::simulate_paste(&spec);
    if let Some(previous) = previous {
        std::thread::sleep(Duration::from_millis(200));
        let _ = clipboard::write_clipboard_text(&previous);
    }
    if keep_open {
        reveal_picker(app, state);
    }
}

fn type_text(app: &tauri::AppHandle, state: &AppState, text: &str, keep_open: bool) -> bool {
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        return false;
    }
    let ok = if wait {
        std::thread::sleep(Duration::from_millis(70));
        platform::simulate_type(text)
    } else {
        platform::simulate_type(text)
    };
    if ok && keep_open {
        reveal_picker(app, state);
    }
    ok
}

fn reveal_picker(app: &tauri::AppHandle, state: &AppState) {
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    show_window(app);
}

fn show_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Some(state) = app.try_state::<AppState>() {
        *state.picker_open.lock().expect("picker_open") = true;
    }
    let _ = window.set_always_on_top(true);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_window(app: &tauri::AppHandle, state: &AppState) {
    *state.picker_open.lock().expect("picker_open") = false;
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
        .inner_size(340.0, 300.0)
        .resizable(false)
        .build()
        .map(|window| window.set_focus());
}

pub(crate) fn register_from_clipboard(app: &tauri::AppHandle, state: &AppState) {
    let previous = clipboard::peek_text();
    let spec = {
        let app_name = platform::capture_foreground()
            .map(|fg| platform::app_and_title(&fg).0)
            .unwrap_or_default();
        state.settings.lock().expect("settings").copy_for(&app_name)
    };
    let _ = platform::simulate_copy(&spec);
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
    reset_n(state);
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
            let n_start = settings.n();
            let last_position = Mutex::new(settings.window().map(|geom| Point {
                x: geom.x,
                y: geom.y,
            }));
            app.manage(AppState {
                store: Mutex::new(store),
                settings: Mutex::new(settings),
                foreground: Mutex::new(None),
                last_position,
                paste_serial: Mutex::new(n_start),
                last_sh: Mutex::new(None),
                picker_open: Mutex::new(false),
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
            undo_change,
            redo_change,
            put_item,
            update_item,
            set_tag,
            set_app_tag,
            set_pinned,
            move_pins,
            split_items,
            get_context,
            merge_items,
            clone_items,
            substitute_items,
            dedup_items,
            sort_items,
            drop_paths,
            edit_external,
            hide_picker,
            paste_items,
            copy_items,
            expand_items,
            expand_text,
            list_tags,
            open_target,
            get_help,
            help_topics,
            export_items,
            import_items,
            clear_unpinned,
            paste_script,
            paste_echo,
            paste_last_script,
            nudge_window,
            resize_window,
            get_shortcuts,
            set_shortcuts,
            pause_shortcuts,
            resume_shortcuts,
            get_keymaps,
            set_keymaps,
            open_settings_file,
            apply_set,
            apply_n
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
            if let tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::Destroyed,
                ..
            } = &event
            {
                if label == "settings" {
                    let shortcuts = app
                        .state::<AppState>()
                        .settings
                        .lock()
                        .expect("settings")
                        .shortcuts()
                        .clone();
                    let _ = shortcuts::resume(app, &shortcuts);
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
