use crate::actions;
use crate::chord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_REGISTER: &str = "Control+Digit4";
pub const DEFAULT_SHOW: &str = "Control+Digit7";

const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Shortcuts {
    pub register: String,
    pub show: String,
    #[serde(default = "bool_true")]
    pub quick_paste: bool,
    #[serde(default = "default_expand")]
    pub expand: String,
}

fn bool_true() -> bool {
    true
}

pub const DEFAULT_EXPAND: &str = "Control+Shift+KeyH";

fn default_expand() -> String {
    DEFAULT_EXPAND.to_string()
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            register: DEFAULT_REGISTER.to_string(),
            show: DEFAULT_SHOW.to_string(),
            quick_paste: true,
            expand: default_expand(),
        }
    }
}

/// 論理ピクセルの内側サイズと、物理ピクセルの外側位置。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowGeom {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyMap {
    pub lhs: String,
    pub rhs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyMaps {
    #[serde(default = "default_leader")]
    pub leader: String,
    #[serde(default)]
    pub maps: Vec<KeyMap>,
}

fn default_leader() -> String {
    " ".to_string()
}

impl Default for KeyMaps {
    fn default() -> Self {
        Self {
            leader: default_leader(),
            maps: Vec::new(),
        }
    }
}

/// 前面アプリへ送るコピー／貼り付け。無い欄は `ctrl+c` / `ctrl+v`。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetKeys {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paste: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    shortcuts: Shortcuts,
    #[serde(default)]
    window: Option<WindowGeom>,
    #[serde(default)]
    keymaps: KeyMaps,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    target_keys: BTreeMap<String, TargetKeys>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    vars: BTreeMap<String, String>,
    #[serde(default = "default_n", skip_serializing_if = "is_default_n")]
    n: u32,
}

pub struct Settings {
    path: PathBuf,
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
    keymaps: KeyMaps,
    target_keys: BTreeMap<String, TargetKeys>,
    vars: BTreeMap<String, String>,
    n: u32,
}

pub enum SetCommand {
    List,
    Paste(String),
    Var { name: String, value: String },
}

pub enum NCommand {
    Show,
    Set(u32),
}

fn default_n() -> u32 {
    1
}

fn is_default_n(n: &u32) -> bool {
    *n == 1
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let loaded = read_file(&path);
        Self {
            path,
            shortcuts: loaded.shortcuts,
            window: loaded.window,
            keymaps: loaded.keymaps,
            target_keys: loaded.target_keys,
            vars: loaded.vars,
            n: loaded.n,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn persist(&self) {
        self.save();
    }

    pub fn reload(&mut self) -> bool {
        let Ok(data) = fs::read_to_string(&self.path) else {
            return false;
        };
        let Ok(file) = serde_json::from_str::<SettingsFile>(&data) else {
            return false;
        };
        let loaded = from_file(file);
        self.shortcuts = loaded.shortcuts;
        self.window = loaded.window;
        self.keymaps = loaded.keymaps;
        self.target_keys = loaded.target_keys;
        self.vars = loaded.vars;
        self.n = loaded.n;
        true
    }

    pub fn vars(&self) -> &BTreeMap<String, String> {
        &self.vars
    }

    pub fn set_var(&mut self, name: &str, value: String) -> bool {
        if !valid_var_name(name) {
            return false;
        }
        if value.is_empty() {
            self.vars.remove(name);
        } else {
            self.vars.insert(name.to_string(), value);
        }
        self.save();
        true
    }

    pub fn n(&self) -> u32 {
        self.n
    }

    pub fn set_n(&mut self, n: u32) {
        self.n = n;
        self.save();
    }

    pub fn format_vars(&self) -> String {
        if self.vars.is_empty() {
            return "変数はない".to_string();
        }
        self.vars
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn shortcuts(&self) -> &Shortcuts {
        &self.shortcuts
    }

    pub fn window(&self) -> Option<WindowGeom> {
        self.window
    }

    pub fn keymaps(&self) -> &KeyMaps {
        &self.keymaps
    }

    pub fn copy_for(&self, app: &str) -> String {
        self.target_keys
            .get(&app.to_lowercase())
            .and_then(|row| row.copy.clone())
            .unwrap_or_else(|| chord::DEFAULT_COPY.to_string())
    }

    pub fn paste_for(&self, app: &str) -> String {
        self.target_keys
            .get(&app.to_lowercase())
            .and_then(|row| row.paste.clone())
            .unwrap_or_else(|| chord::DEFAULT_PASTE.to_string())
    }

    pub fn set_target_paste(&mut self, app: &str, spec: &str) -> bool {
        let app = app.trim().to_lowercase();
        let Some(parsed) = chord::parse(spec) else {
            return false;
        };
        if app.is_empty() {
            return false;
        }
        self.target_keys.entry(app).or_default().paste = Some(chord::display(&parsed));
        self.save();
        true
    }

    pub fn set_shortcuts(&mut self, shortcuts: Shortcuts) {
        self.shortcuts = shortcuts;
        self.save();
    }

    pub fn set_window(&mut self, window: WindowGeom) {
        self.window = Some(window);
        self.save();
    }

    pub fn set_keymaps(&mut self, keymaps: KeyMaps) {
        self.keymaps = sanitize_keymaps(keymaps);
        self.save();
    }

    fn save(&self) {
        let _ = write_file(
            &self.path,
            &self.shortcuts,
            self.window,
            &self.keymaps,
            &self.target_keys,
            &self.vars,
            self.n,
        );
    }
}

struct Loaded {
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
    keymaps: KeyMaps,
    target_keys: BTreeMap<String, TargetKeys>,
    vars: BTreeMap<String, String>,
    n: u32,
}

fn empty_loaded() -> Loaded {
    Loaded {
        shortcuts: Shortcuts::default(),
        window: None,
        keymaps: KeyMaps::default(),
        target_keys: BTreeMap::new(),
        vars: BTreeMap::new(),
        n: default_n(),
    }
}

fn read_file(path: &Path) -> Loaded {
    let Ok(data) = fs::read_to_string(path) else {
        return empty_loaded();
    };
    let Ok(file) = serde_json::from_str::<SettingsFile>(&data) else {
        return empty_loaded();
    };
    from_file(file)
}

fn from_file(file: SettingsFile) -> Loaded {
    let shortcuts = Shortcuts {
        register: usable(file.shortcuts.register, DEFAULT_REGISTER),
        show: usable(file.shortcuts.show, DEFAULT_SHOW),
        quick_paste: file.shortcuts.quick_paste,
        expand: usable_expand(file.shortcuts.expand),
    };
    Loaded {
        shortcuts,
        window: file.window,
        keymaps: sanitize_keymaps(file.keymaps),
        target_keys: sanitize_target_keys(file.target_keys),
        vars: sanitize_vars(file.vars),
        n: file.n,
    }
}

pub fn parse_set(rest: &str) -> Option<SetCommand> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(SetCommand::List);
    }
    if let Some(spec) = rest.strip_prefix("paste") {
        let spec = spec.trim();
        if spec.starts_with('=') {
            return None;
        }
        if spec.is_empty() {
            return None;
        }
        return Some(SetCommand::Paste(spec.to_string()));
    }
    let eq = rest.find('=')?;
    let name = rest[..eq].trim();
    if !valid_var_name(name) {
        return None;
    }
    let raw = rest[eq + 1..].trim();
    let value = parse_set_value(raw)?;
    Some(SetCommand::Var {
        name: name.to_string(),
        value,
    })
}

pub fn parse_n(rest: &str) -> Option<NCommand> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(NCommand::Show);
    }
    if !rest.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    rest.parse().ok().map(NCommand::Set)
}

fn parse_set_value(raw: &str) -> Option<String> {
    if let Some(inner) = unquote(raw, '"') {
        return Some(inner);
    }
    if let Some(inner) = unquote(raw, '\'') {
        return Some(inner);
    }
    Some(raw.to_string())
}

fn unquote(raw: &str, quote: char) -> Option<String> {
    let mut chars = raw.chars();
    if chars.next() != Some(quote) {
        return None;
    }
    let rest: String = chars.collect();
    let end = rest.find(quote)?;
    if !rest[end + quote.len_utf8()..].trim().is_empty() {
        return None;
    }
    Some(rest[..end].to_string())
}

fn valid_var_name(name: &str) -> bool {
    if name == "paste" {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn usable(value: String, fallback: &str) -> String {
    if actions::parse(&value).is_ok() {
        value
    } else {
        fallback.to_string()
    }
}

fn usable_expand(value: String) -> String {
    if value.is_empty() {
        String::new()
    } else {
        usable(value, DEFAULT_EXPAND)
    }
}

fn usable_leader(value: String) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() == 1 && !('1'..='9').contains(&chars[0]) {
        value
    } else {
        default_leader()
    }
}

fn sanitize_keymaps(keymaps: KeyMaps) -> KeyMaps {
    let mut seen = std::collections::HashSet::new();
    let maps = keymaps
        .maps
        .into_iter()
        .filter(|entry| {
            !entry.lhs.is_empty()
                && !entry.rhs.is_empty()
                && !entry.lhs.chars().all(|ch| ('1'..='9').contains(&ch))
                && seen.insert(entry.lhs.clone())
        })
        .collect();
    KeyMaps {
        leader: usable_leader(keymaps.leader),
        maps,
    }
}

fn sanitize_target_keys(raw: BTreeMap<String, TargetKeys>) -> BTreeMap<String, TargetKeys> {
    let mut out = BTreeMap::new();
    for (app, mut row) in raw {
        let app = app.trim().to_lowercase();
        if app.is_empty() {
            continue;
        }
        row.copy = row
            .copy
            .and_then(|spec| chord::parse(&spec).map(|parsed| chord::display(&parsed)));
        row.paste = row
            .paste
            .and_then(|spec| chord::parse(&spec).map(|parsed| chord::display(&parsed)));
        if row.copy.is_none() && row.paste.is_none() {
            continue;
        }
        out.insert(app, row);
    }
    out
}

fn sanitize_vars(raw: BTreeMap<String, String>) -> BTreeMap<String, String> {
    raw.into_iter()
        .filter(|(name, _)| valid_var_name(name))
        .collect()
}

fn write_file(
    path: &Path,
    shortcuts: &Shortcuts,
    window: Option<WindowGeom>,
    keymaps: &KeyMaps,
    target_keys: &BTreeMap<String, TargetKeys>,
    vars: &BTreeMap<String, String>,
    n: u32,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = SettingsFile {
        version: CURRENT_VERSION,
        shortcuts: shortcuts.clone(),
        window,
        keymaps: keymaps.clone(),
        target_keys: target_keys.clone(),
        vars: vars.clone(),
        n,
    };
    fs::write(path, serde_json::to_string_pretty(&file)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::new_id;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hataclip-settings-{}-{}-{name}.json",
            std::process::id(),
            new_id()
        ))
    }

    #[test]
    fn missing_file_yields_defaults() {
        let path = temp_path("missing");
        let _ = fs::remove_file(&path);
        let settings = Settings::load(path);
        assert_eq!(settings.shortcuts(), &Shortcuts::default());
        assert_eq!(settings.window(), None);
        assert_eq!(settings.keymaps(), &KeyMaps::default());
        assert_eq!(settings.n(), 1);
        assert_eq!(settings.copy_for("putty"), "ctrl+c");
        assert_eq!(settings.paste_for("putty"), "ctrl+v");
    }

    #[test]
    fn broken_json_yields_defaults() {
        let path = temp_path("broken");
        fs::write(&path, "{not json").unwrap();
        let settings = Settings::load(path.clone());
        assert_eq!(settings.shortcuts(), &Shortcuts::default());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn unusable_shortcut_falls_back_to_default() {
        let path = temp_path("unusable");
        fs::write(
            &path,
            r#"{"version":1,"shortcuts":{"register":"Control+KeyM","show":"Digit7"}}"#,
        )
        .unwrap();
        let settings = Settings::load(path.clone());
        assert_eq!(settings.shortcuts().register, "Control+KeyM");
        assert_eq!(settings.shortcuts().show, DEFAULT_SHOW);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn round_trip_save_and_load() {
        let path = temp_path("round");
        let _ = fs::remove_file(&path);
        let mut settings = Settings::load(path.clone());
        settings.set_shortcuts(Shortcuts {
            register: "Alt+KeyC".to_string(),
            show: "Alt+KeyV".to_string(),
            quick_paste: false,
            expand: "Control+Shift+KeyH".to_string(),
        });
        settings.set_window(WindowGeom {
            x: 10,
            y: 20,
            width: 400,
            height: 300,
        });

        let reloaded = Settings::load(path.clone());
        assert_eq!(reloaded.shortcuts().register, "Alt+KeyC");
        assert_eq!(reloaded.shortcuts().show, "Alt+KeyV");
        assert_eq!(reloaded.shortcuts().expand, "Control+Shift+KeyH");
        assert_eq!(reloaded.n(), 1);
        assert_eq!(
            reloaded.window(),
            Some(WindowGeom {
                x: 10,
                y: 20,
                width: 400,
                height: 300,
            })
        );
        settings.set_n(100);
        assert_eq!(Settings::load(path.clone()).n(), 100);
        settings.set_n(1);
        assert_eq!(Settings::load(path.clone()).n(), 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn keymaps_round_trip_and_keep_shortcuts() {
        let path = temp_path("maps");
        let _ = fs::remove_file(&path);
        let mut settings = Settings::load(path.clone());
        settings.set_keymaps(KeyMaps {
            leader: ",".into(),
            maps: vec![KeyMap {
                lhs: "<leader>*".into(),
                rhs: "<cmd>bullet".into(),
            }],
        });
        settings.set_shortcuts(Shortcuts::default());
        let reloaded = Settings::load(path.clone());
        assert_eq!(reloaded.keymaps().leader, ",");
        assert_eq!(reloaded.keymaps().maps[0].lhs, "<leader>*");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn target_keys_round_trip_and_drop_invalid() {
        let path = temp_path("targets");
        let _ = fs::remove_file(&path);
        let mut settings = Settings::load(path.clone());
        assert!(settings.set_target_paste("PuTTY", "Shift+Insert"));
        assert!(!settings.set_target_paste("putty", "nope"));
        assert!(!settings.set_target_paste("", "ctrl+v"));
        let reloaded = Settings::load(path.clone());
        assert_eq!(reloaded.paste_for("putty"), "shift+insert");
        assert_eq!(reloaded.copy_for("putty"), "ctrl+c");
        fs::write(
            &path,
            r#"{"version":1,"shortcuts":{"register":"Control+Digit4","show":"Control+Digit7"},"target_keys":{"WT":{"copy":"ctrl+shift+c","paste":"nope"}}}"#,
        )
        .unwrap();
        assert!(settings.reload());
        assert_eq!(settings.copy_for("wt"), "ctrl+shift+c");
        assert_eq!(settings.paste_for("wt"), "ctrl+v");
        fs::write(&path, "{not json").unwrap();
        assert!(!settings.reload());
        assert_eq!(settings.copy_for("wt"), "ctrl+shift+c");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn parse_set_reads_paste_quotes_and_unquoted() {
        assert!(matches!(parse_set(""), Some(SetCommand::List)));
        assert!(matches!(
            parse_set("paste shift+insert"),
            Some(SetCommand::Paste(spec)) if spec == "shift+insert"
        ));
        assert!(matches!(
            parse_set("a=\"{{date}}\""),
            Some(SetCommand::Var { name, value }) if name == "a" && value == "{{date}}"
        ));
        assert!(matches!(
            parse_set("a=':sh dir'"),
            Some(SetCommand::Var { name, value }) if name == "a" && value == ":sh dir"
        ));
        assert!(matches!(
            parse_set("b={{date}}"),
            Some(SetCommand::Var { name, value }) if name == "b" && value == "{{date}}"
        ));
        assert!(parse_set("paste=\"no\"").is_none());
        assert!(parse_set("paste").is_none());
        assert!(matches!(
            parse_set("a="),
            Some(SetCommand::Var { name, value }) if name == "a" && value.is_empty()
        ));
        assert!(parse_set("1a=x").is_none());
        assert!(matches!(parse_n(""), Some(NCommand::Show)));
        assert!(matches!(parse_n("100"), Some(NCommand::Set(100))));
        assert!(matches!(parse_n(" 0 "), Some(NCommand::Set(0))));
        assert!(parse_n("1a").is_none());
        assert!(parse_n("-1").is_none());
    }

    #[test]
    fn vars_round_trip_and_drop_invalid() {
        let path = temp_path("vars");
        let _ = fs::remove_file(&path);
        let mut settings = Settings::load(path.clone());
        assert!(settings.set_var("a", "{{date}}".into()));
        assert!(!settings.set_var("paste", "no".into()));
        assert_eq!(
            Settings::load(path.clone()).vars().get("a").map(String::as_str),
            Some("{{date}}")
        );
        assert!(settings.set_var("a", String::new()));
        assert!(Settings::load(path.clone()).vars().get("a").is_none());
        fs::write(
            &path,
            r#"{"version":1,"shortcuts":{"register":"Control+Digit4","show":"Control+Digit7"},"vars":{"ok":"1","paste":"x","1bad":"y"}}"#,
        )
        .unwrap();
        assert!(settings.reload());
        assert_eq!(settings.vars().get("ok").map(String::as_str), Some("1"));
        assert!(!settings.vars().contains_key("paste"));
        assert!(!settings.vars().contains_key("1bad"));
        let _ = fs::remove_file(path);
    }
}
