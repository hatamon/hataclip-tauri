//! GUI バイナリだけが読む `--test-*` フラグ。E2E から本番の AppData を汚さずに操作するための入口。
//! `hataclip.exe`（パイプ用の `parse_command`）には触らない。

use crate::store::Item;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestLaunch {
    pub dir: Option<PathBuf>,
    pub reset_db: bool,
    pub inject_history: Option<PathBuf>,
    pub dump_state: Option<PathBuf>,
}

impl TestLaunch {
    /// `--test-dir` が無ければテストモードではない。
    pub fn is_active(&self) -> bool {
        self.dir.is_some()
    }
}

/// `--test-*` を読み、それ以外の引数は素通しする。
/// `--test-dir` が無いのに他の `--test-*` があれば失敗。
/// `--test-dir` が本番の AppData と同じ場所なら失敗。
/// 知らない `--test-*` も失敗。
pub fn parse_test_args(
    args: impl IntoIterator<Item = impl AsRef<str>>,
    production_dir: Option<&Path>,
) -> Result<TestLaunch, String> {
    let mut dir: Option<PathBuf> = None;
    let mut reset_db = false;
    let mut inject_history: Option<PathBuf> = None;
    let mut dump_state: Option<PathBuf> = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        let arg = arg.as_ref().to_string();
        match arg.as_str() {
            "--test-dir" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--test-dir にパスが無い".to_string())?;
                dir = Some(PathBuf::from(value.as_ref()));
            }
            "--test-reset-db" => reset_db = true,
            "--test-inject-history" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--test-inject-history にパスが無い".to_string())?;
                inject_history = Some(PathBuf::from(value.as_ref()));
            }
            "--test-dump-state" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--test-dump-state にパスが無い".to_string())?;
                dump_state = Some(PathBuf::from(value.as_ref()));
            }
            other if other.starts_with("--test-") => {
                return Err(format!("知らないフラグ: {other}"));
            }
            _ => {}
        }
    }

    if dir.is_none() && (reset_db || inject_history.is_some() || dump_state.is_some()) {
        return Err("--test-dir が要る".to_string());
    }

    if let (Some(path), Some(production)) = (&dir, production_dir) {
        if same_path(path, production) {
            return Err("--test-dir に本番のフォルダは渡せない".to_string());
        }
    }

    Ok(TestLaunch {
        dir,
        reset_db,
        inject_history,
        dump_state,
    })
}

fn same_path(a: &Path, b: &Path) -> bool {
    let canon_a = std::fs::canonicalize(a).unwrap_or_else(|_| a.to_path_buf());
    let canon_b = std::fs::canonicalize(b).unwrap_or_else(|_| b.to_path_buf());
    canon_a == canon_b
}

/// `--test-reset-db` と `--test-inject-history` を、ストアを開く前に適用する。
pub fn prepare_data_dir(dir: &Path, launch: &TestLaunch) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|_| "テスト用フォルダを作れない".to_string())?;
    if launch.reset_db {
        let _ = std::fs::remove_file(dir.join("items.json"));
        let _ = std::fs::remove_file(dir.join("settings.json"));
    }
    if let Some(source) = &launch.inject_history {
        inject_history(dir, source)?;
    }
    Ok(())
}

/// `{ "version": 1, "items": [ { "text": "..." } ] }` を読み、`id` など無い項目を埋めて `items.json` に書く。
fn inject_history(dir: &Path, source: &Path) -> Result<(), String> {
    let raw =
        std::fs::read_to_string(source).map_err(|_| "履歴ファイルを読めない".to_string())?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "履歴ファイルの形式が読めない".to_string())?;
    let items = value
        .get("items")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "items が無い".to_string())?;

    let mut normalized = Vec::with_capacity(items.len());
    for item in items {
        let mut obj = item
            .as_object()
            .cloned()
            .ok_or_else(|| "行の形式が違う".to_string())?;
        if !obj.contains_key("text") {
            return Err("text が無い行がある".to_string());
        }
        obj.entry("id".to_string())
            .or_insert_with(|| serde_json::Value::String(crate::store::new_id()));
        obj.entry("tags".to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        obj.entry("pinned".to_string())
            .or_insert_with(|| serde_json::Value::Bool(false));
        obj.entry("contexts".to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        obj.entry("pin_rank".to_string())
            .or_insert_with(|| serde_json::Value::Number(0.into()));
        obj.entry("formula".to_string())
            .or_insert_with(|| serde_json::Value::String(String::new()));
        normalized.push(serde_json::Value::Object(obj));
    }

    let out = serde_json::json!({ "version": 1, "items": normalized });
    let text = serde_json::to_string_pretty(&out).map_err(|_| "書けない".to_string())?;
    std::fs::write(dir.join("items.json"), text).map_err(|_| "書けない".to_string())
}

/// `--test-dump-state` の内容。起動完了と、以後の保存のたびに書く。
#[derive(Debug, Clone, Default)]
pub struct DumpStatus {
    pub ready: bool,
    pub shortcuts_ok: bool,
    pub picker_open: bool,
    pub ui: TestUi,
    pub last_open: Option<String>,
}

/// 一覧のフロントから来るスナップショット。テストだけが読む。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestUi {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub query: String,
    pub selected: Option<String>,
    #[serde(default)]
    pub visible: Vec<String>,
    #[serde(default)]
    pub filtered: Vec<String>,
    #[serde(default)]
    pub preview: String,
    #[serde(default)]
    pub info: String,
    #[serde(default)]
    pub which: Vec<String>,
    #[serde(default)]
    pub context_only: bool,
}

struct DumpSink {
    path: PathBuf,
    status: Arc<Mutex<DumpStatus>>,
    items: Mutex<Vec<Item>>,
}

static DUMP: Mutex<Option<DumpSink>> = Mutex::new(None);

#[derive(Serialize)]
struct DumpItem<'a> {
    text: &'a str,
    tags: &'a [String],
}

#[derive(Serialize)]
struct DumpPicker<'a> {
    open: bool,
    mode: &'a str,
    query: &'a str,
    selected: Option<&'a str>,
    visible: &'a [String],
    filtered: &'a [String],
    preview: &'a str,
    info: &'a str,
    which: &'a [String],
    #[serde(rename = "contextOnly")]
    context_only: bool,
}

#[derive(Serialize)]
struct DumpFile<'a> {
    ready: bool,
    shortcuts: &'static str,
    items: Vec<DumpItem<'a>>,
    clipboard: String,
    picker: DumpPicker<'a>,
    #[serde(rename = "lastOpen")]
    last_open: Option<&'a str>,
}

pub fn attach_dump(path: PathBuf, status: Arc<Mutex<DumpStatus>>) {
    *DUMP.lock().unwrap_or_else(|err| err.into_inner()) = Some(DumpSink {
        path,
        status,
        items: Mutex::new(Vec::new()),
    });
}

pub fn write_dump(path: &Path, status: &DumpStatus, items: &[Item]) {
    let dump = DumpFile {
        ready: status.ready,
        shortcuts: if status.shortcuts_ok { "ok" } else { "failed" },
        items: items
            .iter()
            .map(|item| DumpItem {
                text: &item.text,
                tags: &item.tags,
            })
            .collect(),
        clipboard: crate::clipboard::peek_text().unwrap_or_default(),
        picker: DumpPicker {
            open: status.picker_open,
            mode: &status.ui.mode,
            query: &status.ui.query,
            selected: status.ui.selected.as_deref(),
            visible: &status.ui.visible,
            filtered: &status.ui.filtered,
            preview: &status.ui.preview,
            info: &status.ui.info,
            which: &status.ui.which,
            context_only: status.ui.context_only,
        },
        last_open: status.last_open.as_deref(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&dump) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, json);
    }
}

fn flush_sink() {
    let sink = DUMP.lock().unwrap_or_else(|err| err.into_inner());
    let Some(sink) = sink.as_ref() else {
        return;
    };
    let snapshot = sink.status.lock().unwrap_or_else(|err| err.into_inner()).clone();
    let items = sink.items.lock().unwrap_or_else(|err| err.into_inner()).clone();
    write_dump(&sink.path, &snapshot, &items);
}

/// `Store::save` の末尾から呼ぶフック。呼ぶたびそのときの `status` で書く。
pub fn dump_hook(
    path: PathBuf,
    status: Arc<Mutex<DumpStatus>>,
) -> Arc<dyn Fn(&[Item]) + Send + Sync> {
    Arc::new(move |items: &[Item]| {
        if let Some(sink) = DUMP.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
            *sink.items.lock().unwrap_or_else(|err| err.into_inner()) = items.to_vec();
        }
        let snapshot = status.lock().unwrap_or_else(|err| err.into_inner()).clone();
        write_dump(&path, &snapshot, items);
    })
}

pub fn remember_items(items: &[Item]) {
    if let Some(sink) = DUMP.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
        *sink.items.lock().unwrap_or_else(|err| err.into_inner()) = items.to_vec();
    }
}

pub fn set_ui(ui: TestUi) {
    if let Some(sink) = DUMP.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
        sink.status.lock().unwrap_or_else(|err| err.into_inner()).ui = ui;
    }
    flush_sink();
}

pub fn set_picker_open(open: bool) {
    if let Some(sink) = DUMP.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
        sink.status.lock().unwrap_or_else(|err| err.into_inner()).picker_open = open;
    }
    flush_sink();
}

pub fn note_open(target: &str) {
    if let Some(sink) = DUMP.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
        sink.status
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .last_open = Some(target.to_string());
    }
    flush_sink();
}

static TEST_MODE: AtomicBool = AtomicBool::new(false);

/// 起動時に一度だけ立てる。フックが注入キーを受け入れるかの判定に使う。
pub fn set_active(active: bool) {
    TEST_MODE.store(active, Ordering::Relaxed);
}

pub fn is_active() -> bool {
    TEST_MODE.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "hataclip-test-args-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmp dir");
        dir
    }

    #[test]
    fn no_args_is_not_test_mode() {
        let launch = parse_test_args(Vec::<String>::new(), None).expect("parse");
        assert!(!launch.is_active());
        assert_eq!(launch, TestLaunch::default());
    }

    #[test]
    fn test_dir_alone_is_active() {
        let launch = parse_test_args(["--test-dir", "C:/work"], None).expect("parse");
        assert!(launch.is_active());
        assert_eq!(launch.dir, Some(PathBuf::from("C:/work")));
        assert!(!launch.reset_db);
    }

    #[test]
    fn other_flags_without_test_dir_fail() {
        assert!(parse_test_args(["--test-reset-db"], None).is_err());
        assert!(parse_test_args(["--test-inject-history", "x.json"], None).is_err());
        assert!(parse_test_args(["--test-dump-state", "x.json"], None).is_err());
    }

    #[test]
    fn unknown_test_flag_fails() {
        assert!(parse_test_args(["--test-dir", "x", "--test-nope"], None).is_err());
    }

    #[test]
    fn missing_value_fails() {
        assert!(parse_test_args(["--test-dir"], None).is_err());
    }

    #[test]
    fn non_test_args_pass_through() {
        let launch =
            parse_test_args(["--foo", "bar", "--test-dir", "C:/work"], None).expect("parse");
        assert_eq!(launch.dir, Some(PathBuf::from("C:/work")));
    }

    #[test]
    fn all_flags_parse_together() {
        let launch = parse_test_args(
            [
                "--test-dir",
                "C:/work",
                "--test-reset-db",
                "--test-inject-history",
                "seed.json",
                "--test-dump-state",
                "dump.json",
            ],
            None,
        )
        .expect("parse");
        assert!(launch.reset_db);
        assert_eq!(launch.inject_history, Some(PathBuf::from("seed.json")));
        assert_eq!(launch.dump_state, Some(PathBuf::from("dump.json")));
    }

    #[test]
    fn same_as_production_dir_is_rejected() {
        let dir = tmp_dir("prod");
        let result = parse_test_args(
            ["--test-dir", dir.to_str().expect("utf8")],
            Some(&dir),
        );
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn different_dir_is_accepted_even_if_production_is_set() {
        let prod = tmp_dir("prod2");
        let other = tmp_dir("other2");
        let result = parse_test_args(["--test-dir", other.to_str().expect("utf8")], Some(&prod));
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&prod);
        let _ = std::fs::remove_dir_all(&other);
    }

    #[test]
    fn reset_db_removes_only_the_two_files() {
        let dir = tmp_dir("reset");
        std::fs::write(dir.join("items.json"), "{}").expect("write");
        std::fs::write(dir.join("settings.json"), "{}").expect("write");
        std::fs::write(dir.join("keep.txt"), "keep").expect("write");
        let launch = TestLaunch {
            dir: Some(dir.clone()),
            reset_db: true,
            inject_history: None,
            dump_state: None,
        };
        prepare_data_dir(&dir, &launch).expect("prepare");
        assert!(!dir.join("items.json").exists());
        assert!(!dir.join("settings.json").exists());
        assert!(dir.join("keep.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inject_history_fills_missing_fields() {
        let dir = tmp_dir("inject");
        let seed = dir.join("seed.json");
        std::fs::write(
            &seed,
            r#"{ "version": 1, "items": [ { "text": "keep" }, { "text": "{{date}}" } ] }"#,
        )
        .expect("write");
        let launch = TestLaunch {
            dir: Some(dir.clone()),
            reset_db: false,
            inject_history: Some(seed.clone()),
            dump_state: None,
        };
        prepare_data_dir(&dir, &launch).expect("prepare");
        let written = std::fs::read_to_string(dir.join("items.json")).expect("read");
        let value: serde_json::Value = serde_json::from_str(&written).expect("json");
        let items = value["items"].as_array().expect("array");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["text"], "keep");
        assert!(items[0]["id"].as_str().is_some());
        assert_eq!(items[1]["text"], "{{date}}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inject_history_rejects_missing_items_key() {
        let dir = tmp_dir("inject-bad");
        let seed = dir.join("seed.json");
        std::fs::write(&seed, r#"{ "version": 1 }"#).expect("write");
        let launch = TestLaunch {
            dir: Some(dir.clone()),
            reset_db: false,
            inject_history: Some(seed),
            dump_state: None,
        };
        assert!(prepare_data_dir(&dir, &launch).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_dump_round_trips() {
        let dir = tmp_dir("dump");
        let path = dir.join("state.json");
        let status = DumpStatus {
            ready: true,
            shortcuts_ok: true,
            ..DumpStatus::default()
        };
        let items = vec![Item::new("hello".to_string(), vec!["url".to_string()])];
        write_dump(&path, &status, &items);
        let written = std::fs::read_to_string(&path).expect("read");
        let value: serde_json::Value = serde_json::from_str(&written).expect("json");
        assert_eq!(value["ready"], true);
        assert_eq!(value["shortcuts"], "ok");
        assert_eq!(value["items"][0]["text"], "hello");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
