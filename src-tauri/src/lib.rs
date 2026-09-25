mod actions;
mod cli;
mod chord;
mod clipboard;
mod keys;
mod editor;
mod eval;
mod expr;
mod help;
mod pipe;
mod platform;
mod settings;
mod shell;
mod shortcuts;
mod store;
mod text;
mod tray;

use chrono::Local;
use platform::Point;
use serde::{Deserialize, Serialize};
use settings::{KeyMaps, NCommand, SetCommand, Settings, Shortcuts, WindowGeom};
use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(windows)]
use std::time::Duration;
use store::{Item, Store};
use tauri::{
    Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

#[derive(Clone)]
struct CompleteCycle {
    query: String,
    prefix: String,
    index: usize,
    at: std::time::Instant,
    ids: Vec<String>,
    expand_key: bool,
}

pub(crate) struct AppState {
    store: Mutex<Store>,
    settings: Mutex<Settings>,
    foreground: Mutex<Option<platform::Foreground>>,
    last_position: Mutex<Option<Point>>,
    paste_serial: Mutex<u32>,
    last_sh: Mutex<Option<String>>,
    picker_open: Mutex<bool>,
    complete_cycle: Mutex<Option<CompleteCycle>>,
    last_error: Mutex<Option<String>>,
    pending_expand: Mutex<Option<String>>,
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
    pinned: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Put {
    let mut item = Item::new(text, tags);
    item.pinned = pinned.unwrap_or(false);
    state
        .store
        .lock()
        .expect("store")
        .insert_relative(anchor_id.as_deref(), above, item.clone());
    Put {
        items: view_keeping(&state, &[item.id.clone()]),
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
fn font_px(state: tauri::State<'_, AppState>) -> u32 {
    state.settings.lock().expect("settings").font_px()
}

#[tauri::command]
fn bump_font(delta: i32, state: tauri::State<'_, AppState>) -> u32 {
    state.settings.lock().expect("settings").bump_font(delta)
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
fn filter_items(
    ids: Vec<String>,
    script: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Item>, String> {
    let script = script.trim();
    if script.is_empty() || ids.is_empty() {
        return Ok(view(&state));
    }
    let store = state.store.lock().expect("store");
    let mut updates = Vec::new();
    for id in &ids {
        let Some(item) = store.get(id) else {
            return Ok(view(&state));
        };
        let out = shell::run_script_with_stdin(script, Some(&item.text))
            .map_err(|err| shell::command_error(&err))?;
        updates.push((id.clone(), out));
    }
    drop(store);
    let formula = format!("!! {script}");
    state
        .store
        .lock()
        .expect("store")
        .rewrite_with_formula(&updates, &formula);
    Ok(view(&state))
}

#[tauri::command]
fn dedup_items(state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").dedup();
    view(&state)
}

#[tauri::command]
fn swap_grab(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state.store.lock().expect("store").swap_grab(&ids);
    view(&state)
}

#[tauri::command]
fn colon_sample() -> String {
    clipboard::peek_text().unwrap_or_default()
}

#[tauri::command]
fn preview_colon(expr: String, dot: String, clip: String) -> Option<String> {
    pipe::colon_preview(&expr, &dot, &clip)
}

#[tauri::command]
fn set_formula(id: String, formula: String, state: tauri::State<'_, AppState>) -> Vec<Item> {
    state
        .store
        .lock()
        .expect("store")
        .set_formula(&id, formula);
    view(&state)
}

/// 行に残した式をもう一度実行して本文を置き換える。失敗した行はそのまま。
#[tauri::command]
fn rerun_formula(
    ids: Vec<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Vec<Item> {
    let mut updates = Vec::new();
    for id in &ids {
        let Some(item) = state.store.lock().expect("store").get(id).cloned() else {
            continue;
        };
        let formula = item.formula.trim().to_string();
        if formula.is_empty() {
            continue;
        }
        if let Some(script) = formula.strip_prefix("!! ") {
            if let Ok(out) = shell::run_script_with_stdin(script, Some(&item.text)) {
                if out != item.text {
                    updates.push((id.clone(), out));
                }
            }
            continue;
        }
        let pipe::PasteBody::Run(script) = pipe::classify(&formula) else {
            continue;
        };
        let row_ids = if script.uses_selection {
            vec![id.clone()]
        } else {
            Vec::new()
        };
        let ops: Vec<PipeOp> = script
            .ops
            .into_iter()
            .map(|op| PipeOp {
                kind: op.kind,
                arg: op.arg,
                selection_stdin: op.selection_stdin,
            })
            .collect();
        let mut added = Vec::new();
        if let Ok((text, _)) = walk_pipe(&app, &state, &row_ids, &ops, None, &mut added) {
            if !added.is_empty() {
                let mut store = state.store.lock().expect("store");
                for (body, formula) in added {
                    let tags = text::auto_tags(&body);
                    let mut item = Item::new(body, tags);
                    item.formula = formula;
                    store.insert(item);
                }
            }
            if !text.is_empty() && text != item.text {
                updates.push((id.clone(), text));
            }
        }
    }
    if !updates.is_empty() {
        state
            .store
            .lock()
            .expect("store")
            .replace_texts(&updates);
    }
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
    complete: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let next = Shortcuts {
        register,
        show,
        quick_paste,
        expand,
        complete,
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
        Some(SetCommand::Copy(spec)) => {
            let app = foreground_app(&state);
            let _ = state
                .settings
                .lock()
                .expect("settings")
                .set_target_copy(&app, &spec);
            Ok(None)
        }
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
        Some(SetCommand::Step(name)) => {
            if !state.settings.lock().expect("settings").arm_step(&name) {
                return Err("数字だけ".into());
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

/// 複数行まとめて貼るときは separator でつなぐ。format は `:format` のときだけ真。
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
    to_clipboard: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> bool {
    let to_clipboard = to_clipboard.unwrap_or(false);
    if let Some(text) = resolved {
        return paste_resolved_text(
            &app,
            &state,
            &ids,
            &text,
            keep_open,
            format,
            prefix.as_deref(),
            to_clipboard,
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
        to_clipboard,
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
    to_clipboard: bool,
) -> bool {
    let mut text = if format {
        text::format_for_paste(text)
    } else {
        text.to_string()
    };
    if let Some(prefix) = prefix {
        text = text::prefix_lines(&text, prefix);
    }
    if to_clipboard {
        if clipboard::write_clipboard_text(&text) {
            clear_error(state);
            return true;
        }
        note_error(state, "クリップボードに書けない");
        return false;
    }
    if let Some(key) = current_context(state) {
        state.store.lock().expect("store").record_context(ids, &key);
    }
    let ok = paste_text(app, state, &text, keep_open);
    let _ = app.emit("items-changed", view(state));
    bump_n(state);
    bump_step_vars(state, ids);
    if !keep_open {
        reset_n(state);
    }
    ok
}

#[tauri::command]
fn copy_items(ids: Vec<String>, state: tauri::State<'_, AppState>) -> bool {
    match expand_ids(&state, &ids, true, HashMap::new()) {
        Some(text) => {
            if clipboard::write_clipboard_text(&text) {
                clear_error(&state);
                true
            } else {
                note_error(&state, "クリップボードに書けない");
                false
            }
        }
        None => {
            note_error(&state, "展開できない");
            false
        }
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
    platform::open_target(&text)
}

fn note_error(state: &AppState, message: &str) {
    let message = message.trim();
    if message.is_empty() {
        return;
    }
    *state.last_error.lock().expect("last_error") = Some(message.to_string());
}

fn clear_error(state: &AppState) {
    *state.last_error.lock().expect("last_error") = None;
}

fn mark_paste(state: &AppState, ok: bool) -> bool {
    if ok {
        clear_error(state);
    } else {
        note_error(state, "貼れない");
    }
    ok
}

fn finish<T>(state: &AppState, result: Result<T, String>) -> Result<T, String> {
    match &result {
        Ok(_) => clear_error(state),
        Err(error) => note_error(state, error),
    }
    result
}

#[tauri::command]
fn last_error(state: tauri::State<'_, AppState>) -> String {
    state
        .last_error
        .lock()
        .expect("last_error")
        .clone()
        .unwrap_or_default()
}

#[tauri::command]
fn remember_error(message: String, state: tauri::State<'_, AppState>) {
    note_error(&state, &message);
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipeOp {
    kind: String,
    #[serde(default)]
    arg: String,
    #[serde(default)]
    selection_stdin: bool,
}

fn render_formula(ops: &[PipeOp]) -> String {
    let ops: Vec<pipe::Op> = ops
        .iter()
        .map(|op| pipe::Op {
            kind: op.kind.clone(),
            arg: op.arg.clone(),
            selection_stdin: op.selection_stdin,
        })
        .collect();
    pipe::render_ops(&ops)
}

fn pipe_local(text: &str, kind: &str, arg: &str) -> String {
    match kind {
        "quote" => {
            let (prefix, suffix) = text::split_quote_arg(arg);
            text::affix_lines(text, prefix, suffix)
        }
        "format" => text::format_for_paste(text),
        "join" => text.lines().collect::<Vec<_>>().join(arg),
        "camel" | "pascal" | "snake" | "kebab" | "upper" | "lower" => text::recase(text, kind),
        "sub" => text::substitute_literal(text, arg),
        _ => text.to_string(),
    }
}

fn pipe_bodies(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    raw: bool,
) -> Result<Vec<String>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    if raw {
        let store = state.store.lock().expect("store");
        let rows: Vec<Item> = ids.iter().filter_map(|id| store.get(id).cloned()).collect();
        if rows.len() != ids.len() {
            return Err("行がない".into());
        }
        return Ok(rows.into_iter().map(|item| item.text).collect());
    }
    let sel = if has_sel(state, ids) {
        capture_selection(app, state)
    } else {
        String::new()
    };
    let ctx = expand_context(state, sel, HashMap::new());
    let store = state.store.lock().expect("store");
    let rows: Vec<Item> = ids.iter().filter_map(|id| store.get(id).cloned()).collect();
    drop(store);
    if rows.len() != ids.len() {
        return Err("行がない".into());
    }
    let mut parts = Vec::new();
    for item in &rows {
        let ops = resolve_item(item, &ctx, false).ok_or("展開できない")?;
        parts.push(text::flatten_ops(&ops));
    }
    Ok(parts)
}

fn pipe_text(parts: &[String], sep: &str) -> String {
    parts.join(sep)
}

/// `|` でつないだ段を左から適用し、末尾の行き先へ出す。`show` のときだけ文字列を返す。
#[tauri::command]
fn run_pipe(
    ids: Vec<String>,
    ops: Vec<PipeOp>,
    sink: String,
    set_name: Option<String>,
    keep_open: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    finish(
        &state,
        execute_pipe(
        &app,
        &state,
        &ids,
        &ops,
        &sink,
        set_name.as_deref(),
        keep_open.unwrap_or(false),
        None,
    ),
    )
}

fn walk_pipe(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    ops: &[PipeOp],
    seed: Option<String>,
    added: &mut Vec<(String, String)>,
) -> Result<(String, bool), String> {
    let mut text: Option<String> = seed;
    let mut raw = false;
    let mut used_selection = false;
    let mut each_at = None;
    for (index, op) in ops.iter().enumerate() {
        if op.kind == "each" {
            each_at = Some(index);
            break;
        }
        match op.kind.as_str() {
            "raw" => {
                if text.is_none() {
                    raw = true;
                }
            }
            "dot" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(&app, &state, &ids, true)?, "\n"));
                }
            }
            "add" => {
                let Some(current) = text.as_deref() else {
                    return Err("段がない".into());
                };
                if !current.is_empty() {
                    added.push((current.to_string(), render_formula(&ops[..index])));
                }
            }
            "clip" => match text.as_deref() {
                Some(current) => {
                    if !current.is_empty() && !clipboard::write_clipboard_text(current) {
                        return Err("クリップボードに書けない".into());
                    }
                }
                None => {
                    text = Some(clipboard::peek_text().unwrap_or_default());
                }
            },
            "sel" => {
                if text.is_none() {
                    text = Some(capture_pipe_selection(&app, &state)?);
                }
            }
            "echo" => {
                let value = expr::eval_with(&op.arg, &var_map(&state)).ok_or("計算できない")?;
                text = Some(expr::format_number(value));
                raw = false;
            }
            "sh" => {
                let stdin = if let Some(current) = text.take() {
                    Some(current)
                } else if op.selection_stdin {
                    used_selection = true;
                    Some(pipe_text(&pipe_bodies(&app, &state, &ids, true)?, "\n"))
                } else {
                    None
                };
                let output = shell::run_script_with_stdin(&op.arg, stdin.as_deref())
                    .map_err(|err| shell::command_error(&err))?;
                *state.last_sh.lock().expect("last_sh") = Some(op.arg.clone());
                text = Some(output);
                raw = false;
            }
            "json" | "xml" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(&app, &state, &ids, raw)?, "\n"));
                    raw = false;
                }
                if let Some(current) = text.as_mut() {
                    let pretty = if op.kind == "json" {
                        text::pretty_json(current)
                    } else {
                        text::pretty_xml(current)
                    };
                    match pretty {
                        Some(next) => *current = next,
                        None => return Err("読めない".into()),
                    }
                }
            }
            "put" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
                    raw = false;
                }
                let Some((pointer, value)) = op.arg.split_once('\u{1}') else {
                    return Err("段が違う".into());
                };
                if let Some(current) = text.as_mut() {
                    *current = text::json_put(current, pointer, value).ok_or("書けない")?;
                }
            }
            "diff" | "only" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
                    raw = false;
                }
                let other = match op.arg.as_str() {
                    "." => {
                        used_selection = true;
                        pipe_text(&pipe_bodies(app, state, ids, true)?, "\n")
                    }
                    "clip" => clipboard::peek_text().unwrap_or_default(),
                    _ => return Err("段が違う".into()),
                };
                if let Some(current) = text.as_mut() {
                    let next = if op.kind == "diff" {
                        text::line_diff(current, &other)
                    } else {
                        text::only_lines(current, &other)
                    };
                    *current = next.ok_or("同じ")?;
                }
            }
            "filter" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
                    raw = false;
                }
                if let Some(current) = text.as_mut() {
                    *current = text::filter_lines(current, &op.arg).ok_or("当たらない")?;
                }
            }
            "split" | "col" | "get" => {
                if text.is_none() {
                    used_selection = true;
                    text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
                    raw = false;
                }
                if let Some(current) = text.as_mut() {
                    let next = match op.kind.as_str() {
                        "split" => Some(text::split_fields(current, &op.arg)),
                        "col" => {
                            let index = op.arg.parse::<i64>().unwrap_or(0);
                            text::take_column(current, index)
                        }
                        "get" => text::json_at(current, &op.arg),
                        _ => None,
                    };
                    match next {
                        Some(value) => *current = value,
                        None => return Err("抜けない".into()),
                    }
                }
            }
            "quote" | "format" | "join" | "sub" | "camel" | "pascal" | "snake" | "kebab" | "upper" | "lower" => {
                if text.is_none() {
                    used_selection = true;
                    let sep = if op.kind == "join" { op.arg.as_str() } else { "\n" };
                    text = Some(pipe_text(&pipe_bodies(&app, &state, &ids, raw)?, sep));
                    raw = false;
                    if op.kind == "join" {
                        continue;
                    }
                }
                if let Some(current) = text.as_mut() {
                    *current = pipe_local(current, &op.kind, &op.arg);
                }
            }
            _ => return Err("段が違う".into()),
        }
    }
    if text.is_none() {
        used_selection = true;
        text = Some(pipe_text(&pipe_bodies(&app, &state, &ids, raw)?, "\n"));
    }
    if let Some(index) = each_at {
        if text.is_none() {
            used_selection = true;
            text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
        }
        let body = text.unwrap_or_default();
        let mut kept = Vec::new();
        for line in body.lines() {
            match walk_pipe(
                app,
                state,
                ids,
                &ops[index + 1..],
                Some(line.to_string()),
                added,
            ) {
                Ok((out, used)) => {
                    if used {
                        used_selection = true;
                    }
                    kept.push(out);
                }
                Err(_) => {}
            }
        }
        if kept.is_empty() {
            return Err("当たらない".into());
        }
        return Ok((kept.join("\n"), used_selection));
    }
    if text.is_none() {
        used_selection = true;
        text = Some(pipe_text(&pipe_bodies(app, state, ids, raw)?, "\n"));
    }
    Ok((text.unwrap_or_default(), used_selection))
}

fn execute_pipe(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    ops: &[PipeOp],
    sink: &str,
    set_name: Option<&str>,
    keep_open: bool,
    seed: Option<String>,
) -> Result<Option<String>, String> {
    if ops.is_empty() {
        return Err("段がない".into());
    }
    if seed.is_none() && !cfg!(windows) && ops.iter().any(|op| op.kind == "sel") {
        return Err("選択を取れない".into());
    }
    let mut added = Vec::new();
    let (text, used_selection) = walk_pipe(app, state, ids, ops, seed, &mut added)?;
    if !added.is_empty() {
        {
            let mut store = state.store.lock().expect("store");
            for (body, formula) in added {
                let tags = text::auto_tags(&body);
                let mut item = Item::new(body, tags);
                item.formula = formula;
                store.insert(item);
            }
        }
        let _ = app.emit("items-changed", view(state));
    }
    let pasted_ids: Vec<String> = if used_selection { ids.to_vec() } else { Vec::new() };
    match sink {
        "paste" => {
            if text.is_empty() && !ops.iter().any(|op| op.kind == "sub") {
                return Ok(None);
            }
            if !paste_resolved_text(app, state, &pasted_ids, &text, keep_open, false, None, false) {
                let message = state
                    .last_error
                    .lock()
                    .expect("last_error")
                    .clone()
                    .unwrap_or_else(|| "貼れない".into());
                return Err(message);
            }
            Ok(None)
        }
        "clip" => {
            if text.is_empty() {
                return Ok(None);
            }
            if !clipboard::write_clipboard_text(&text) {
                return Err("クリップボードに書けない".into());
            }
            Ok(None)
        }
        "add" => {
            if text.is_empty() {
                return Ok(None);
            }
            let tags = text::auto_tags(&text);
            let mut item = Item::new(text, tags);
            item.formula = render_formula(ops);
            state.store.lock().expect("store").insert(item);
            let _ = app.emit("items-changed", view(&state));
            Ok(None)
        }
        "set" => {
            let name = set_name.unwrap_or("").to_string();
            if !state.settings.lock().expect("settings").set_var(&name, text) {
                return Err("名前が違う".into());
            }
            Ok(None)
        }
        "open" => {
            let target = text.trim();
            if target.starts_with("http://") || target.starts_with("https://") || text::looks_like_path(target)
            {
                platform::open_target(target);
            }
            Ok(None)
        }
        "show" => Ok(Some(text)),
        "log" => {
            if text.is_empty() {
                return Ok(None);
            }
            let vars = var_map(state);
            let Some(path) = log_destination(
                set_name.unwrap_or(""),
                vars.get("defaultLogFileName").map(String::as_str),
            ) else {
                return Ok(None);
            };
            let _ = append_log(&path, &text);
            Ok(None)
        }
        _ => Err("行き先が違う".into()),
    }
}

#[tauri::command]
fn paste_script(
    script: String,
    to_clipboard: Option<bool>,
    stdin: Option<String>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let script = script.trim().to_string();
    if script.is_empty() {
        return Ok(());
    }
    let output = shell::run_script_with_stdin(&script, stdin.as_deref())
        .map_err(|err| shell::command_error(&err))?;
    *state.last_sh.lock().expect("last_sh") = Some(script);
    if to_clipboard.unwrap_or(false) {
        if !clipboard::write_clipboard_text(&output) {
            return Err("クリップボードに書けない".into());
        }
        return Ok(());
    }
    paste_text(&app, &state, &output, false);
    Ok(())
}

#[tauri::command]
fn paste_echo(
    expr: String,
    to_clipboard: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Ok(());
    }
    let vars = var_map(&state);
    let value = expr::eval_with(expr, &vars).ok_or("計算できない")?;
    let text = expr::format_number(value);
    if to_clipboard.unwrap_or(false) {
        if !clipboard::write_clipboard_text(&text) {
            return Err("クリップボードに書けない".into());
        }
        return Ok(());
    }
    paste_text(&app, &state, &text, false);
    Ok(())
}

#[tauri::command]
fn paste_last_script(
    to_clipboard: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some(script) = state.last_sh.lock().expect("last_sh").clone() else {
        return Ok(());
    };
    paste_script(script, to_clipboard, None, app, state)
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
    match apply_row_pipe(app, state, &item.id, &item.text, false) {
        RowPipe::Text => {}
        RowPipe::Show(text) => {
            reveal_picker(app, state);
            let _ = app.emit("pipe-shown", text);
            return;
        }
        RowPipe::Noop | RowPipe::Done => return,
    }
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
        false,
    );
}

#[derive(Serialize)]
struct PipePaste {
    status: String,
    text: Option<String>,
}

enum RowPipe {
    Text,
    Noop,
    Done,
    Show(String),
}

/// 本文がパイプなら実行する。テンプレートなら `Text`。
fn apply_row_pipe(
    app: &tauri::AppHandle,
    state: &AppState,
    id: &str,
    text: &str,
    keep_open: bool,
) -> RowPipe {
    let script = match pipe::classify(text) {
        pipe::PasteBody::Text => return RowPipe::Text,
        pipe::PasteBody::Bad => return RowPipe::Noop,
        pipe::PasteBody::Run(script) => script,
    };
    let show = script.sink == "show";
    let ids = if script.uses_selection {
        vec![id.to_string()]
    } else {
        Vec::new()
    };
    let ops: Vec<PipeOp> = script
        .ops
        .into_iter()
        .map(|op| PipeOp {
            kind: op.kind,
            arg: op.arg,
            selection_stdin: op.selection_stdin,
        })
        .collect();
    match finish(
        state,
        execute_pipe(
        app,
        state,
        &ids,
        &ops,
        &script.sink,
        script.set_name.as_deref(),
        keep_open,
        None,
    ),
    ) {
        Ok(Some(text)) if show => RowPipe::Show(text),
        Ok(_) => RowPipe::Done,
        Err(_) => RowPipe::Noop,
    }
}

#[tauri::command]
fn paste_pipe_item(
    id: String,
    keep_open: Option<bool>,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> PipePaste {
    let text = state
        .store
        .lock()
        .expect("store")
        .get(&id)
        .map(|item| item.text.clone());
    let Some(text) = text else {
        return PipePaste {
            status: "noop".into(),
            text: None,
        };
    };
    match apply_row_pipe(&app, &state, &id, &text, keep_open.unwrap_or(false)) {
        RowPipe::Text => PipePaste {
            status: "text".into(),
            text: None,
        },
        RowPipe::Noop => PipePaste {
            status: "noop".into(),
            text: None,
        },
        RowPipe::Done => PipePaste {
            status: "done".into(),
            text: None,
        },
        RowPipe::Show(text) => PipePaste {
            status: "show".into(),
            text: Some(text),
        },
    }
}

pub(crate) fn paste_from_selection(app: &tauri::AppHandle, state: &AppState) {
    if *state.picker_open.lock().expect("picker_open") {
        return;
    }
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    if let Some(cycle) = pending_cycle(state).filter(|cycle| cycle.expand_key) {
        if !advance_complete(app, state, &cycle) {
            *state.complete_cycle.lock().expect("complete") = None;
            note_error(state, "当たらない");
        }
        return;
    }
    let captured = capture_front_text(state);
    let (text, from_line) = if let Some(text) = captured {
        (text, false)
    } else {
        match select_line_start(state) {
            LinePick::Empty => {
                note_error(state, "選択が空");
                return;
            }
            LinePick::Newline => {
                note_error(state, "行頭までの選択に改行がある");
                return;
            }
            LinePick::Text(text) => (text, true),
        }
    };
    if let Some((prefix, search)) = completion_span(&text, from_line) {
        if search.trim().is_empty() {
            note_error(state, "選択が空");
            return;
        }
        begin_complete(app, state, text, &search, &prefix, true);
        return;
    }
    if ask_shell(app, state, &text) {
        return;
    }
    expand_captured(app, state, &text);
}

fn ask_shell(app: &tauri::AppHandle, state: &AppState, text: &str) -> bool {
    let last_sh = state.last_sh.lock().expect("last_sh").clone();
    let prompts = shell_prompts(text, last_sh.as_deref());
    if prompts.is_empty() {
        return false;
    }
    *state.pending_expand.lock().expect("pending_expand") = Some(text.to_string());
    show_window(app);
    let _ = app.emit("sh-confirm", prompts.join("\n"));
    true
}

fn shell_prompts(text: &str, last_sh: Option<&str>) -> Vec<String> {
    let mut prompts = Vec::new();
    let first = text.trim().lines().next().unwrap_or("").trim();
    if first == ":@" {
        if let Some(script) = last_sh.filter(|script| !script.is_empty()) {
            prompts.push(script.to_string());
        }
    }
    let script = match pipe::headed_pipe(text) {
        pipe::Headed::Run { script, .. } => Some(script),
        _ => match pipe::solo_pipe(text) {
            pipe::Solo::Run(script) => Some(script),
            _ => None,
        },
    };
    if let Some(script) = script {
        for op in script.ops {
            if op.kind == "sh" && !op.arg.trim().is_empty() {
                prompts.push(op.arg);
            }
        }
    }
    prompts.extend(shell::sh_args(text));
    prompts
}

fn expand_captured(app: &tauri::AppHandle, state: &AppState, text: &str) {
    if apply_headed_pipe(app, state, text) {
        return;
    }
    if apply_solo_pipe(app, state, &text) {
        return;
    }
    let ctx = expand_context(state, String::new(), HashMap::new());
    let last_sh = state.last_sh.lock().expect("last_sh").clone();
    let ran = if shell::has_sh_token(text) {
        match shell::apply_sh(text, true) {
            Ok(out) => Some(out),
            Err(err) => {
                note_error(state, &shell::command_error(&err));
                return;
            }
        }
    } else {
        None
    };
    let body = ran.as_deref().unwrap_or(text);
    let Some(out) = eval::resolve_selection(body, &ctx, last_sh.as_deref()).or_else(|| {
        ran.filter(|out| out != text)
    }) else {
        note_error(state, "展開できない");
        return;
    };
    let ops = text::take_type_ops(&out);
    if !text::has_keys(&ops) && text::flatten_ops(&ops).is_empty() {
        note_error(state, "展開が空");
        return;
    }
    remember_last_sh(text, state);
    play_resolved(app, state, ops, false, false);
}

#[tauri::command]
fn confirm_selection_expand(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    let text = state.pending_expand.lock().expect("pending_expand").take();
    hide_window(&app, &state);
    let Some(text) = text else {
        return;
    };
    expand_captured(&app, &state, &text);
}

#[tauri::command]
fn cancel_selection_expand(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    *state.pending_expand.lock().expect("pending_expand") = None;
    hide_window(&app, &state);
}

/// 1行目が `:` のパイプなら、2行目以降を流れにして実行する。扱ったら真。
fn apply_headed_pipe(app: &tauri::AppHandle, state: &AppState, text: &str) -> bool {
    let (script, flow) = match pipe::headed_pipe(text) {
        pipe::Headed::Skip => return false,
        pipe::Headed::Noop => return true,
        pipe::Headed::Run { script, flow } => (script, flow),
    };
    play_script(app, state, script, Some(flow));
    true
}

/// 1行で `sh` か `echo` が結果を作るパイプ。引用符の外の `|` は段。扱ったら真。
fn apply_solo_pipe(app: &tauri::AppHandle, state: &AppState, text: &str) -> bool {
    let script = match pipe::solo_pipe(text) {
        pipe::Solo::Skip => return false,
        pipe::Solo::Noop => return true,
        pipe::Solo::Run(script) => script,
    };
    play_script(app, state, script, None);
    true
}

fn play_script(
    app: &tauri::AppHandle,
    state: &AppState,
    script: pipe::Script,
    flow: Option<String>,
) {
    let show = script.sink == "show";
    let ops: Vec<PipeOp> = script
        .ops
        .into_iter()
        .map(|op| PipeOp {
            kind: op.kind,
            arg: op.arg,
            selection_stdin: op.selection_stdin,
        })
        .collect();
    match finish(
        state,
        execute_pipe(
        app,
        state,
        &[],
        &ops,
        &script.sink,
        script.set_name.as_deref(),
        false,
        flow,
    ),
    ) {
        Ok(Some(shown)) if show => {
            reveal_picker(app, state);
            let _ = app.emit("pipe-shown", shown);
        }
        Ok(_) => {}
        Err(_) => {}
    }
}

/// 前面の選択語で履歴を補完して貼る。テンプレートは一覧の Enter と同じく展開する。一覧は出さない。
pub(crate) fn complete_from_selection(app: &tauri::AppHandle, state: &AppState) {
    if *state.picker_open.lock().expect("picker_open") {
        return;
    }
    *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    if let Some(cycle) = pending_cycle(state) {
        if !advance_complete(app, state, &cycle) {
            *state.complete_cycle.lock().expect("complete") = None;
            note_error(state, "当たらない");
        }
        return;
    }
    let Some(query) = capture_front_text(state).filter(|text| !text.trim().is_empty()) else {
        note_error(state, "選択が空");
        return;
    };
    begin_complete(app, state, query.clone(), &query, "", false);
}

fn pending_cycle(state: &AppState) -> Option<CompleteCycle> {
    let now = std::time::Instant::now();
    state
        .complete_cycle
        .lock()
        .expect("complete")
        .clone()
        .filter(|cycle| now.duration_since(cycle.at) < std::time::Duration::from_secs(2))
}

fn begin_complete(
    app: &tauri::AppHandle,
    state: &AppState,
    identity: String,
    search: &str,
    prefix: &str,
    expand_key: bool,
) {
    let ids = completion_ids(state, search);
    let Some(index) = paste_completion_from(app, state, &ids, 0, &identity, prefix) else {
        note_error(state, "当たらない");
        return;
    };
    *state.complete_cycle.lock().expect("complete") = Some(CompleteCycle {
        query: identity,
        prefix: prefix.to_string(),
        index,
        at: std::time::Instant::now(),
        ids,
        expand_key,
    });
}

enum LinePick {
    Empty,
    Newline,
    Text(String),
}

#[cfg(windows)]
fn select_line_start(state: &AppState) -> LinePick {
    if !platform::simulate_chord("shift+home") {
        return LinePick::Empty;
    }
    std::thread::sleep(std::time::Duration::from_millis(50));
    match capture_front_text(state) {
        Some(text) if text.contains('\n') || text.contains('\r') => LinePick::Newline,
        Some(text) => LinePick::Text(text),
        None => LinePick::Empty,
    }
}

#[cfg(not(windows))]
fn select_line_start(_state: &AppState) -> LinePick {
    LinePick::Empty
}

/// 先頭の `/`、または行頭まで自動選択した行の最後の `/` から補完する。
fn completion_span(text: &str, from_line: bool) -> Option<(String, String)> {
    if from_line {
        let at = text.rfind('/')?;
        return Some((text[..at].to_string(), text[at + '/'.len_utf8()..].to_string()));
    }
    let search = text.trim().strip_prefix('/')?;
    Some((String::new(), search.to_string()))
}

fn advance_complete(app: &tauri::AppHandle, state: &AppState, cycle: &CompleteCycle) -> bool {
    let Some(index) = completion_index(
        state,
        &cycle.ids,
        cycle.index.wrapping_add(1),
        &cycle.query,
    ) else {
        return false;
    };
    #[cfg(windows)]
    {
        if !platform::simulate_chord("ctrl+z") {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(80));
        let Some(back) = capture_front_text(state) else {
            return false;
        };
        if back != cycle.query {
            return false;
        }
    }
    let Some(index) = paste_completion_from(app, state, &cycle.ids, index, &cycle.query, &cycle.prefix) else {
        return false;
    };
    *state.complete_cycle.lock().expect("complete") = Some(CompleteCycle {
        query: cycle.query.clone(),
        prefix: cycle.prefix.clone(),
        index,
        at: std::time::Instant::now(),
        ids: cycle.ids.clone(),
        expand_key: cycle.expand_key,
    });
    true
}

/// `start` から順に展開して貼る。空や対象外は飛ばす。全部だめなら `None`。
fn paste_completion_from(
    app: &tauri::AppHandle,
    state: &AppState,
    ids: &[String],
    start: usize,
    sel: &str,
    prefix: &str,
) -> Option<usize> {
    if ids.is_empty() {
        return None;
    }
    let start = start % ids.len();
    for offset in 0..ids.len() {
        let index = (start + offset) % ids.len();
        match deliver_completion(app, state, &ids[index], sel, prefix) {
            Some(true) => return Some(index),
            Some(false) => return None,
            None => continue,
        }
    }
    None
}

fn deliver_completion(
    app: &tauri::AppHandle,
    state: &AppState,
    id: &str,
    sel: &str,
    prefix: &str,
) -> Option<bool> {
    let item = {
        state
            .store
            .lock()
            .expect("store")
            .get(id)
            .cloned()
    }?;
    let ctx = expand_context(state, sel.to_string(), HashMap::new());
    let mut ops = completion_ops(&item, &ctx)?;
    if !prefix.is_empty() {
        ops.insert(0, text::PasteOp::Text(prefix.to_string()));
    }
    let ok = play_resolved(app, state, ops, false, false);
    if ok {
        bump_n(state);
        bump_step_vars(state, &[id.to_string()]);
    }
    Some(ok)
}

fn completion_blocked(tags: &[String], text: &str) -> bool {
    if tags.iter().any(|tag| {
        matches!(tag.as_str(), "secret" | "run" | "confirm" | "grab")
    }) {
        return true;
    }
    if !text::ask_names(text).is_empty() || !text::pick_specs(text).is_empty() {
        return true;
    }
    !matches!(pipe::classify(text), pipe::PasteBody::Text)
}

fn completion_ops(item: &Item, ctx: &text::Expand) -> Option<Vec<text::PasteOp>> {
    if completion_blocked(&item.tags, &item.text) {
        return None;
    }
    let ops = resolve_item(item, ctx, false)?;
    if text::flatten_ops(&ops).is_empty() && !text::has_keys(&ops) {
        return None;
    }
    Some(ops)
}

/// 展開して空でない次の候補。無ければ `Ctrl+Z` しない。
fn completion_index(state: &AppState, ids: &[String], start: usize, sel: &str) -> Option<usize> {
    if ids.is_empty() {
        return None;
    }
    let start = start % ids.len();
    for offset in 0..ids.len() {
        let index = (start + offset) % ids.len();
        let item = {
            state
                .store
                .lock()
                .expect("store")
                .get(&ids[index])
                .cloned()
        };
        let Some(item) = item else {
            continue;
        };
        let ctx = expand_context(state, sel.to_string(), HashMap::new());
        if completion_ops(&item, &ctx).is_some() {
            return Some(index);
        }
    }
    None
}

fn completion_ids(state: &AppState, query: &str) -> Vec<String> {
    let context = current_context(state);
    let store = state.store.lock().expect("store");
    let rows: Vec<Item> = store::ordered(store.list(), context.as_deref())
        .into_iter()
        .filter(|item| !completion_blocked(&item.tags, &item.text))
        .collect();
    let rows: Vec<(String, String, Vec<String>)> = rows
        .into_iter()
        .map(|item| (item.id, item.text, item.tags))
        .collect();
    text::search_ids(&rows, query)
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

fn capture_front_text(state: &AppState) -> Option<String> {
    #[cfg(windows)]
    {
        let previous = clipboard::peek_text();
        let spec = copy_spec(state);
        let _ = platform::simulate_copy(&spec);
        std::thread::sleep(Duration::from_millis(200));
        let captured = clipboard::peek_text();
        match captured {
            Some(text) if previous.as_ref() != Some(&text) && !text.is_empty() => Some(text),
            _ => None,
        }
    }
    #[cfg(not(windows))]
    {
        let _ = state;
        clipboard::peek_text()
    }
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
    to_clipboard: bool,
) -> bool {
    let was_open = *state.picker_open.lock().expect("picker_open");
    let needs_grab = !raw && rows_have_tag(state, ids, "grab");
    let needs_sel = !raw && !quiet && has_sel(state, ids);
    let sel = if needs_grab {
        capture_grab_input(app, state)
    } else if needs_sel {
        capture_selection(app, state)
    } else {
        String::new()
    };
    let grab_input = sel.clone();
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
        let grabbed = if item.tags.iter().any(|tag| tag == "grab") && ctx.is_some() {
            let Some(filled) = text::grab_fill(&item.text, &grab_input) else {
                if was_open {
                    show_window(app);
                }
                return false;
            };
            let mut copy = item.clone();
            copy.text = filled;
            copy.tags.retain(|tag| tag != "grab");
            Some(copy)
        } else {
            None
        };
        let resolved = if let Some(ctx) = ctx.as_ref() {
                    match resolve_item(grabbed.as_ref().unwrap_or(item), ctx, false) {
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
        ops = map_text_ops(ops, |text| text::prefix_lines(text, prefix));
    }
    if paste_parts.is_empty() {
        if to_clipboard {
            if was_open {
                show_window(app);
            }
            return false;
        }
        if !logged {
            if was_open {
                show_window(app);
            }
            return false;
        }
        if let Some(key) = current_context(state) {
            state.store.lock().expect("store").record_context(ids, &key);
        }
        hide_window(app, state);
        let _ = app.emit("items-changed", view(state));
        if keep_open {
            reveal_picker(app, state);
        }
        bump_n(state);
        if ctx.is_some() {
            bump_step_vars(state, ids);
        }
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
    if to_clipboard {
        return clipboard::write_clipboard_text(&text::flatten_ops(&ops));
    }
    if let Some(key) = current_context(state) {
        state
            .store
            .lock()
            .expect("store")
            .record_context(ids, &key);
    }
    let ok = play_resolved(app, state, ops, keep_open, typed);
    if ok {
        let _ = app.emit("items-changed", view(state));
        bump_n(state);
        if ctx.is_some() {
            bump_step_vars(state, ids);
        }
    } else if was_open {
        reveal_picker(app, state);
    }
    if !keep_open {
        reset_n(state);
    }
    ok
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

fn bump_step_vars(state: &AppState, ids: &[String]) {
    let app = foreground_app(state);
    let vars = var_map(state);
    let focus = platform::focused_control();
    let names = {
        let store = state.store.lock().expect("store");
        let mut names = Vec::new();
        for id in ids {
            let Some(item) = store.get(id) else {
                continue;
            };
            for name in text::referenced_vars(
                &item.text,
                &text::WhenEnv {
                    app: &app,
                    focus: &focus,
                    vars: &vars,
                },
            ) {
                if !names.iter().any(|existing| existing == &name) {
                    names.push(name);
                }
            }
        }
        names
    };
    if names.is_empty() {
        return;
    }
    state.settings.lock().expect("settings").bump_steps(&names);
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
        focus: platform::focused_control(),
        now,
        answers,
        vars: var_map(state),
        tags: tag_map(state),
    }
}

fn rows_have_tag(state: &AppState, ids: &[String], tag: &str) -> bool {
    let store = state.store.lock().expect("store");
    ids.iter().any(|id| {
        store
            .get(id)
            .map_or(false, |item| item.tags.iter().any(|existing| existing == tag))
    })
}

fn capture_grab_input(app: &tauri::AppHandle, state: &AppState) -> String {
    #[cfg(windows)]
    {
        capture_selection(app, state)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, state);
        clipboard::peek_text().unwrap_or_default()
    }
}

fn has_sel(state: &AppState, ids: &[String]) -> bool {
    let app = foreground_app(state);
    let vars = var_map(state);
    let focus = platform::focused_control();
    let store = state.store.lock().expect("store");
    ids.iter().any(|id| {
        store.get(id).map_or(false, |item| {
            text::has_sel_token_in(
                &item.text,
                &text::WhenEnv {
                    app: &app,
                    focus: &focus,
                    vars: &vars,
                },
            )
        })
    })
}

#[cfg(windows)]
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
    std::thread::sleep(Duration::from_millis(200));
    let captured = clipboard::peek_text();
    match captured {
        Some(text) if previous.as_ref() != Some(&text) && !text.is_empty() => text,
        _ => String::new(),
    }
}

#[cfg(not(windows))]
fn capture_selection(_app: &tauri::AppHandle, _state: &AppState) -> String {
    String::new()
}

/// パイプの `sel`。前面へ Ctrl+C を送り、200ms 後を読む。クリップボードは戻さない。
fn capture_pipe_selection(app: &tauri::AppHandle, state: &AppState) -> Result<String, String> {
    #[cfg(windows)]
    {
        let previous = clipboard::peek_text();
        let was_open = *state.picker_open.lock().expect("picker_open");
        if was_open {
            hide_window(app, state);
            let foreground = *state.foreground.lock().expect("foreground");
            if let Some(foreground) = foreground.as_ref() {
                let _ = platform::restore_foreground(foreground);
            }
            std::thread::sleep(Duration::from_millis(70));
        }
        let _ = platform::simulate_copy(&copy_spec(state));
        std::thread::sleep(Duration::from_millis(200));
        let captured = clipboard::peek_text();
        let text = match captured {
            Some(text) if previous.as_ref() != Some(&text) && !text.is_empty() => text,
            _ => String::new(),
        };
        if was_open {
            reveal_picker(app, state);
        }
        if text.is_empty() {
            return Err("選択が空".into());
        }
        Ok(text)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, state);
        Err("選択を取れない".into())
    }
}

fn resolve_item(item: &Item, ctx: &text::Expand, force_sh: bool) -> Option<Vec<text::PasteOp>> {
    let run = item.tags.iter().any(|tag| tag == "run");
    let file = item.tags.iter().any(|tag| tag == "file");
    if run && !shell::has_sh_token(&item.text) {
        let expanded = text::expand_template(&item.text, ctx);
        return Some(vec![text::PasteOp::Text(
            shell::run_script(&expanded).ok()?,
        )]);
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
        Some(vec![text::PasteOp::Text(text::flatten_ops(&ops))])
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
            if text.is_empty() {
                note_error(state, "展開が空");
                return false;
            }
            return type_text(app, state, &text, keep_open);
        }
        paste_text(app, state, &text, keep_open)
    } else {
        play_ops(app, state, &ops, keep_open, typed)
    }
}

#[cfg(windows)]
fn play_ops(
    app: &tauri::AppHandle,
    state: &AppState,
    ops: &[text::PasteOp],
    keep_open: bool,
    typed: bool,
) -> bool {
    let spec = paste_spec(state);
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        return mark_paste(state, false);
    }
    if wait {
        std::thread::sleep(Duration::from_millis(70));
    }
    let mut did_paste = false;
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
            return mark_paste(state, false);
        }
    }
    if did_paste {
        std::thread::sleep(Duration::from_millis(200));
    }
    if keep_open {
        reveal_picker(app, state);
    }
    mark_paste(state, true)
}

#[cfg(not(windows))]
fn play_ops(
    _app: &tauri::AppHandle,
    state: &AppState,
    _ops: &[text::PasteOp],
    _keep_open: bool,
    _typed: bool,
) -> bool {
    mark_paste(state, false)
}

/// `:log` の出力先。引数が空なら変数 `defaultLogFileName`。無ければ何もしない。
pub(crate) fn log_destination(explicit: &str, fallback: Option<&str>) -> Option<String> {
    let explicit = explicit.trim();
    if !explicit.is_empty() {
        return Some(explicit.to_string());
    }
    fallback
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(str::to_string)
}

#[tauri::command]
fn log_selection(
    ids: Vec<String>,
    path: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let vars = var_map(&state);
    let Some(path) = log_destination(&path, vars.get("defaultLogFileName").map(String::as_str)) else {
        return Ok(());
    };
    if ids.is_empty() {
        return Ok(());
    }
    let parts = pipe_bodies(&app, &state, &ids, false).unwrap_or_default();
    if parts.len() != ids.len() {
        return Ok(());
    }
    let text = parts.join("\n");
    if text.is_empty() {
        return Ok(());
    }
    let _ = append_log(&path, &text);
    Ok(())
}

/// 既存ファイルの最初の改行。見つからなければ CRLF（新規ファイルも同じ）。
fn existing_log_newline(path: &std::path::Path) -> &'static str {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return "\r\n",
    };
    let mut buf = [0u8; 8192];
    let mut prev_cr = false;
    loop {
        let n = match std::io::Read::read(&mut file, &mut buf) {
            Ok(0) => return "\r\n",
            Ok(n) => n,
            Err(_) => return "\r\n",
        };
        for &byte in &buf[..n] {
            if byte == b'\n' {
                return if prev_cr { "\r\n" } else { "\n" };
            }
            prev_cr = byte == b'\r';
        }
    }
}

/// 本文の改行を `nl` に揃え、末尾にも同じ改行を付ける。
fn log_bytes(text: &str, nl: &str) -> Vec<u8> {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str(nl);
            }
            '\n' => out.push_str(nl),
            other => out.push(other),
        }
    }
    if !out.ends_with(nl) {
        out.push_str(nl);
    }
    out.into_bytes()
}

pub(crate) fn append_log(path: &str, text: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let bytes = log_bytes(text, existing_log_newline(path));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    file.write_all(&bytes)?;
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

#[cfg_attr(not(windows), allow(dead_code))]
fn copy_spec(state: &AppState) -> String {
    let app = foreground_app(state);
    state.settings.lock().expect("settings").copy_for(&app)
}

#[cfg_attr(not(windows), allow(dead_code))]
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

#[cfg(windows)]
fn paste_text(app: &tauri::AppHandle, state: &AppState, text: &str, keep_open: bool) -> bool {
    if !clipboard::write_clipboard_text(text) {
        note_error(state, "クリップボードに書けない");
        return false;
    }
    let spec = paste_spec(state);
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        if keep_open {
            reveal_picker(app, state);
        }
        return mark_paste(state, false);
    }
    if wait {
        std::thread::sleep(Duration::from_millis(70));
    }
    let ok = platform::simulate_paste(&spec);
    std::thread::sleep(Duration::from_millis(200));
    if keep_open {
        reveal_picker(app, state);
    }
    mark_paste(state, ok)
}

#[cfg(not(windows))]
fn paste_text(app: &tauri::AppHandle, state: &AppState, text: &str, _keep_open: bool) -> bool {
    let ok = clipboard::write_clipboard_text(text);
    let _ = yield_target(app, state, false);
    if ok {
        mark_paste(state, true)
    } else {
        note_error(state, "クリップボードに書けない");
        false
    }
}

#[cfg(windows)]
fn type_text(app: &tauri::AppHandle, state: &AppState, text: &str, keep_open: bool) -> bool {
    let wait = *state.picker_open.lock().expect("picker_open");
    if !yield_target(app, state, keep_open) {
        return mark_paste(state, false);
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
    mark_paste(state, ok)
}

#[cfg(not(windows))]
fn type_text(_app: &tauri::AppHandle, state: &AppState, _text: &str, _keep_open: bool) -> bool {
    mark_paste(state, false)
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
    let Some(text) = capture_register_text(state) else {
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

#[cfg(windows)]
fn capture_register_text(state: &AppState) -> Option<String> {
    let spec = {
        let app_name = platform::capture_foreground()
            .map(|fg| platform::app_and_title(&fg).0)
            .unwrap_or_default();
        state.settings.lock().expect("settings").copy_for(&app_name)
    };
    let _ = platform::simulate_copy(&spec);
    std::thread::sleep(Duration::from_millis(70));
    clipboard::peek_text().filter(|text| !text.trim().is_empty())
}

#[cfg(not(windows))]
fn capture_register_text(_state: &AppState) -> Option<String> {
    clipboard::peek_text().filter(|text| !text.trim().is_empty())
}

pub(crate) fn show_picker(app: &tauri::AppHandle, state: &AppState) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let already = *state.picker_open.lock().expect("picker_open");
    if !already {
        *state.foreground.lock().expect("foreground") = platform::capture_foreground();
    }
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

    state.store.lock().expect("store").reload_if_newer();
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
pub fn run_cli(expr: &str, stdin: Option<&str>) -> i32 {
    cli::run(expr, stdin)
}

fn watch_items(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            let Some(state) = app.try_state::<AppState>() else {
                break;
            };
            let changed = state.store.lock().expect("store").reload_if_newer();
            if !changed {
                continue;
            }
            let next = view(&state);
            drop(state);
            let _ = app.emit("items-changed", next);
        }
    });
}

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
                complete_cycle: Mutex::new(None),
                last_error: Mutex::new(None),
                pending_expand: Mutex::new(None),
            });
            tray::setup(app)?;
            watch_items(app.handle().clone());
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
            filter_items,
            dedup_items,
            swap_grab,
            set_formula,
            colon_sample,
            preview_colon,
            rerun_formula,
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
            last_error,
            remember_error,
            confirm_selection_expand,
            cancel_selection_expand,
            export_items,
            import_items,
            clear_unpinned,
            run_pipe,
            paste_pipe_item,
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
            apply_n,
            font_px,
            bump_font,
            log_selection
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

    fn completion_ctx() -> text::Expand {
        text::Expand {
            date: "2026/09/20".into(),
            time: "10:54".into(),
            clip: "CLIP".into(),
            sel: "SEL".into(),
            n: 3,
            uuid: "uuid-here".into(),
            user: "hatamon".into(),
            host: "pc".into(),
            app: "code".into(),
            front: "TODO.md".into(),
            focus: "Edit".into(),
            now: Local::now(),
            answers: HashMap::new(),
            vars: HashMap::new(),
            tags: HashMap::new(),
        }
    }

    #[test]
    fn completion_expands_date_and_drops_nop() {
        let ctx = completion_ctx();
        let item = Item::new("報告 {{date}} {{nop: 仕事}}".into(), Vec::new());
        let ops = completion_ops(&item, &ctx).unwrap();
        assert_eq!(text::flatten_ops(&ops), "報告 2026/09/20 {{nop: 仕事}}");
        let nop = completion_ops(&Item::new("{{nop: x}}".into(), Vec::new()), &ctx).unwrap();
        assert_eq!(text::flatten_ops(&nop), "{{nop: x}}");
        assert!(completion_ops(&Item::new("{{ask:名前}}".into(), Vec::new()), &ctx).is_none());
        assert!(completion_ops(
            &Item::new("{{pick list: a, b}}".into(), Vec::new()),
            &ctx
        )
        .is_none());
        assert!(completion_ops(&Item::new(":sel | upper".into(), Vec::new()), &ctx).is_none());
        assert!(completion_ops(&Item::new("echo hi".into(), vec!["run".into()]), &ctx).is_none());
        assert!(completion_ops(&Item::new("ok".into(), vec!["confirm".into()]), &ctx).is_none());
        assert!(completion_ops(&Item::new("ok".into(), vec!["grab".into()]), &ctx).is_none());
        assert!(completion_ops(&Item::new("ok".into(), vec!["secret".into()]), &ctx).is_none());
    }

    #[test]
    fn log_uses_the_argument_or_the_default_variable() {
        let mut vars: HashMap<String, String> = HashMap::new();
        assert!(log_destination("", None).is_none());
        vars.insert("defaultLogFileName".into(), "  notes.log  ".into());
        assert_eq!(
            log_destination("", vars.get("defaultLogFileName").map(String::as_str)).as_deref(),
            Some("notes.log")
        );
        assert_eq!(log_destination(" other.log ", None).as_deref(), Some("other.log"));
        vars.insert("defaultLogFileName".into(), "   ".into());
        assert!(log_destination("", vars.get("defaultLogFileName").map(String::as_str)).is_none());
    }

    #[test]
    fn log_matches_existing_newlines_and_starts_crlf() {
        let dir = std::env::temp_dir().join(format!("hataclip-log-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let crlf = dir.join("crlf.log");
        std::fs::write(&crlf, "old\r\n").unwrap();
        append_log(crlf.to_str().unwrap(), "a\nb").unwrap();
        assert_eq!(std::fs::read(&crlf).unwrap(), b"old\r\na\r\nb\r\n");
        let lf = dir.join("lf.log");
        std::fs::write(&lf, "old\n").unwrap();
        append_log(lf.to_str().unwrap(), "a\r\nb").unwrap();
        assert_eq!(std::fs::read(&lf).unwrap(), b"old\na\nb\n");
        let fresh = dir.join("new.log");
        append_log(fresh.to_str().unwrap(), "a\nb").unwrap();
        assert_eq!(std::fs::read(&fresh).unwrap(), b"a\r\nb\r\n");
        let long = dir.join("long.log");
        let mut body = vec![b'x'; 9000];
        body.extend_from_slice(b"\r\n");
        std::fs::write(&long, &body).unwrap();
        append_log(long.to_str().unwrap(), "z").unwrap();
        let got = std::fs::read(&long).unwrap();
        assert!(got.ends_with(b"\r\nz\r\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pipe_quote_stacks_prefixes() {
        let once = pipe_local("b\na", "quote", "> ");
        let twice = pipe_local(&once, "quote", "x ");
        assert_eq!(twice, "x > b\nx > a");
        assert_eq!(pipe_local("hello", "quote", "> \u{1}<"), "> hello<");
        assert_eq!(pipe_local("> hello<", "quote", "> \u{1}<"), "> hello<");
    }
}
