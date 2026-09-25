use crate::actions;
use crate::chord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
    #[serde(default = "default_complete")]
    pub complete: String,
}

fn bool_true() -> bool {
    true
}

pub const DEFAULT_EXPAND: &str = "Control+Digit8";
pub const DEFAULT_COMPLETE: &str = "Control+Digit9";

fn default_expand() -> String {
    DEFAULT_EXPAND.to_string()
}

fn default_complete() -> String {
    DEFAULT_COMPLETE.to_string()
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            register: DEFAULT_REGISTER.to_string(),
            show: DEFAULT_SHOW.to_string(),
            quick_paste: true,
            expand: default_expand(),
            complete: default_complete(),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut: Option<String>,
}

enum TargetSlot {
    Copy,
    Paste,
    Home,
    Cut,
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
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    steps: BTreeSet<String>,
    #[serde(default = "default_n", skip_serializing_if = "is_default_n")]
    n: u32,
    #[serde(default = "default_font", skip_serializing_if = "is_default_font")]
    font_px: u32,
}

pub struct Settings {
    path: PathBuf,
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
    keymaps: KeyMaps,
    target_keys: BTreeMap<String, TargetKeys>,
    vars: BTreeMap<String, String>,
    steps: BTreeSet<String>,
    n: u32,
    font_px: u32,
}

pub enum SetCommand {
    List,
    Copy(String),
    Paste(String),
    Home(String),
    Cut(String),
    Var { name: String, value: String },
    Step(String),
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

fn default_font() -> u32 {
    13
}

fn is_default_font(px: &u32) -> bool {
    *px == default_font()
}

pub const FONT_MIN: u32 = 9;
pub const FONT_MAX: u32 = 32;

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
            steps: loaded.steps,
            n: loaded.n,
            font_px: loaded.font_px,
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
        self.steps = loaded.steps;
        self.n = loaded.n;
        self.font_px = loaded.font_px;
        true
    }

    pub fn font_px(&self) -> u32 {
        self.font_px
    }

    /// 一覧の文字を 1px 動かす。端では止まる。
    pub fn bump_font(&mut self, delta: i32) -> u32 {
        let next = (self.font_px as i32 + delta).clamp(FONT_MIN as i32, FONT_MAX as i32) as u32;
        if next != self.font_px {
            self.font_px = next;
            self.save();
        }
        self.font_px
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
            self.steps.remove(name);
        } else {
            if value.parse::<i64>().is_err() {
                self.steps.remove(name);
            }
            self.vars.insert(name.to_string(), value);
        }
        self.save();
        true
    }

    /// 貼って成功したあと 1 増やす。無い変数は 1。数字でなければ付けない。
    pub fn arm_step(&mut self, name: &str) -> bool {
        if !valid_var_name(name) {
            return false;
        }
        match self.vars.get(name).map(String::as_str) {
            None => {
                self.vars.insert(name.to_string(), "1".into());
            }
            Some(value) if value.parse::<i64>().is_ok() => {}
            Some(_) => return false,
        }
        self.steps.insert(name.to_string());
        self.save();
        true
    }

    pub fn bump_steps(&mut self, names: &[String]) {
        let mut changed = false;
        for name in names {
            if !self.steps.contains(name) {
                continue;
            }
            let Some(raw) = self.vars.get(name) else {
                continue;
            };
            let Ok(n) = raw.parse::<i64>() else {
                continue;
            };
            self.vars.insert(name.clone(), n.saturating_add(1).to_string());
            changed = true;
        }
        if changed {
            self.save();
        }
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
            .map(|(name, value)| {
                if self.steps.contains(name) {
                    format!("{name}={value} +1")
                } else {
                    format!("{name}={value}")
                }
            })
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

    pub fn home_for(&self, app: &str) -> String {
        self.target_keys
            .get(&app.to_lowercase())
            .and_then(|row| row.home.clone())
            .unwrap_or_else(|| chord::DEFAULT_HOME.to_string())
    }

    /// まだどの動作も送らない。設定だけ残す。
    #[allow(dead_code)]
    pub fn cut_for(&self, app: &str) -> String {
        self.target_keys
            .get(&app.to_lowercase())
            .and_then(|row| row.cut.clone())
            .unwrap_or_else(|| chord::DEFAULT_CUT.to_string())
    }

    pub fn set_target_copy(&mut self, app: &str, spec: &str) -> bool {
        self.set_target_chord(app, spec, TargetSlot::Copy)
    }

    pub fn set_target_paste(&mut self, app: &str, spec: &str) -> bool {
        self.set_target_chord(app, spec, TargetSlot::Paste)
    }

    pub fn set_target_home(&mut self, app: &str, spec: &str) -> bool {
        self.set_target_chord(app, spec, TargetSlot::Home)
    }

    pub fn set_target_cut(&mut self, app: &str, spec: &str) -> bool {
        self.set_target_chord(app, spec, TargetSlot::Cut)
    }

    fn set_target_chord(&mut self, app: &str, spec: &str, slot: TargetSlot) -> bool {
        let app = app.trim().to_lowercase();
        let Some(parsed) = chord::parse(spec) else {
            return false;
        };
        if app.is_empty() {
            return false;
        }
        let shown = chord::display(&parsed);
        let row = self.target_keys.entry(app).or_default();
        match slot {
            TargetSlot::Copy => row.copy = Some(shown),
            TargetSlot::Paste => row.paste = Some(shown),
            TargetSlot::Home => row.home = Some(shown),
            TargetSlot::Cut => row.cut = Some(shown),
        }
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
            &self.steps,
            self.n,
            self.font_px,
        );
    }
}

struct Loaded {
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
    keymaps: KeyMaps,
    target_keys: BTreeMap<String, TargetKeys>,
    vars: BTreeMap<String, String>,
    steps: BTreeSet<String>,
    n: u32,
    font_px: u32,
}

fn empty_loaded() -> Loaded {
    Loaded {
        shortcuts: Shortcuts::default(),
        window: None,
        keymaps: KeyMaps::default(),
        target_keys: BTreeMap::new(),
        vars: BTreeMap::new(),
        steps: BTreeSet::new(),
        n: default_n(),
        font_px: default_font(),
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
        complete: usable_complete(file.shortcuts.complete),
    };
    let vars = sanitize_vars(file.vars);
    Loaded {
        shortcuts,
        window: file.window,
        keymaps: sanitize_keymaps(file.keymaps),
        target_keys: sanitize_target_keys(file.target_keys),
        steps: sanitize_steps(&file.steps, &vars),
        vars,
        n: file.n,
        font_px: file.font_px.clamp(FONT_MIN, FONT_MAX),
    }
}

pub fn parse_set(rest: &str) -> Option<SetCommand> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Some(SetCommand::List);
    }
    if rest.contains("+=") {
        let (name, amount) = rest.split_once("+=")?;
        let name = name.trim();
        let amount = amount.trim();
        if amount != "1" || !valid_var_name(name) {
            return None;
        }
        return Some(SetCommand::Step(name.to_string()));
    }
    if let Some(parsed) = chord_spec(rest, "copy") {
        return parsed.map(SetCommand::Copy);
    }
    if let Some(parsed) = chord_spec(rest, "paste") {
        return parsed.map(SetCommand::Paste);
    }
    if let Some(parsed) = chord_spec(rest, "home") {
        return parsed.map(SetCommand::Home);
    }
    if let Some(parsed) = chord_spec(rest, "cut") {
        return parsed.map(SetCommand::Cut);
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

/// キーワードそのものなら `Some(None)`。別の名前なら `None`。
fn chord_spec(rest: &str, word: &str) -> Option<Option<String>> {
    let Some(tail) = rest.strip_prefix(word) else {
        return None;
    };
    if tail.is_empty() || tail.starts_with('=') {
        return Some(None);
    }
    if !tail.starts_with(char::is_whitespace) {
        return None;
    }
    let spec = tail.trim();
    if spec.is_empty() {
        return Some(None);
    }
    Some(Some(spec.to_string()))
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
    let text = raw.trim_start();
    let mut chars = text.chars();
    if chars.next() != Some(quote) {
        return None;
    }
    let body: Vec<char> = chars.collect();
    let mut out = String::new();
    let mut escaped = false;
    let mut index = 0;
    while index < body.len() {
        let ch = body[index];
        index += 1;
        if escaped {
            out.push(match ch {
                't' => '\t',
                'n' => '\n',
                other => other,
            });
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            let tail: String = body[index..].iter().collect();
            if !tail.trim().is_empty() {
                return None;
            }
            return Some(out);
        }
        out.push(ch);
    }
    None
}

fn valid_var_name(name: &str) -> bool {
    if name == "paste" || name == "copy" || name == "home" || name == "cut" {
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

fn usable_complete(value: String) -> String {
    if value.is_empty() {
        String::new()
    } else {
        usable(value, DEFAULT_COMPLETE)
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
        row.home = row
            .home
            .and_then(|spec| chord::parse(&spec).map(|parsed| chord::display(&parsed)));
        row.cut = row
            .cut
            .and_then(|spec| chord::parse(&spec).map(|parsed| chord::display(&parsed)));
        if row.copy.is_none() && row.paste.is_none() && row.home.is_none() && row.cut.is_none() {
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

fn sanitize_steps(raw: &BTreeSet<String>, vars: &BTreeMap<String, String>) -> BTreeSet<String> {
    raw.iter()
        .filter(|name| {
            valid_var_name(name)
                && vars
                    .get(name.as_str())
                    .and_then(|value| value.parse::<i64>().ok())
                    .is_some()
        })
        .cloned()
        .collect()
}

fn write_file(
    path: &Path,
    shortcuts: &Shortcuts,
    window: Option<WindowGeom>,
    keymaps: &KeyMaps,
    target_keys: &BTreeMap<String, TargetKeys>,
    vars: &BTreeMap<String, String>,
    steps: &BTreeSet<String>,
    n: u32,
    font_px: u32,
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
        steps: steps.clone(),
        n,
        font_px,
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
        let mut settings = Settings::load(path);
        assert_eq!(settings.shortcuts(), &Shortcuts::default());
        assert_eq!(settings.window(), None);
        assert_eq!(settings.keymaps(), &KeyMaps::default());
        assert_eq!(settings.n(), 1);
        assert_eq!(settings.font_px(), 13);
        assert_eq!(settings.bump_font(1), 14);
        assert_eq!(settings.bump_font(-100), FONT_MIN);
        assert_eq!(settings.bump_font(100), FONT_MAX);
        let again = Settings::load(settings.path().to_path_buf());
        assert_eq!(again.font_px(), FONT_MAX);
        let _ = fs::remove_file(again.path());
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
            complete: "Control+Digit9".to_string(),
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
                rhs: "<cmd>quote".into(),
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
        assert!(settings.set_target_copy("PuTTY", "Ctrl+Shift+C"));
        assert!(settings.set_target_paste("PuTTY", "Shift+Insert"));
        assert!(!settings.set_target_copy("putty", "nope"));
        assert!(!settings.set_target_paste("putty", "nope"));
        assert!(!settings.set_target_copy("", "ctrl+c"));
        assert!(!settings.set_target_paste("", "ctrl+v"));
        assert!(settings.set_target_home("PuTTY", "ctrl+shift+home"));
        assert!(settings.set_target_cut("PuTTY", "ctrl+shift+x"));
        assert!(!settings.set_target_home("putty", "nope"));
        let reloaded = Settings::load(path.clone());
        assert_eq!(reloaded.paste_for("putty"), "shift+insert");
        assert_eq!(reloaded.copy_for("putty"), "ctrl+shift+c");
        assert_eq!(reloaded.home_for("putty"), "ctrl+shift+home");
        assert_eq!(reloaded.cut_for("putty"), "ctrl+shift+x");
        assert_eq!(reloaded.home_for("other"), "shift+home");
        assert_eq!(reloaded.cut_for("other"), "ctrl+x");
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
            parse_set("copy ctrl+shift+c"),
            Some(SetCommand::Copy(spec)) if spec == "ctrl+shift+c"
        ));
        assert!(matches!(
            parse_set("paste shift+insert"),
            Some(SetCommand::Paste(spec)) if spec == "shift+insert"
        ));
        assert!(matches!(
            parse_set("a=\"{{date}}\""),
            Some(SetCommand::Var { name, value }) if name == "a" && value == "{{date}}"
        ));
        assert!(matches!(
            parse_set(r#"a="say \"hi\"""#),
            Some(SetCommand::Var { name, value }) if name == "a" && value == "say \"hi\""
        ));
        assert!(matches!(
            parse_set(r#"a="a\tb\nc""#),
            Some(SetCommand::Var { name, value }) if name == "a" && value == "a\tb\nc"
        ));
        assert!(matches!(
            parse_set("a=':sh dir'"),
            Some(SetCommand::Var { name, value }) if name == "a" && value == ":sh dir"
        ));
        assert!(matches!(
            parse_set("b={{date}}"),
            Some(SetCommand::Var { name, value }) if name == "b" && value == "{{date}}"
        ));
        assert!(parse_set("copy=\"no\"").is_none());
        assert!(parse_set("copy").is_none());
        assert!(parse_set("paste=\"no\"").is_none());
        assert!(parse_set("paste").is_none());
        assert!(matches!(
            parse_set("home shift+home"),
            Some(SetCommand::Home(spec)) if spec == "shift+home"
        ));
        assert!(matches!(
            parse_set("cut ctrl+x"),
            Some(SetCommand::Cut(spec)) if spec == "ctrl+x"
        ));
        assert!(parse_set("home").is_none());
        assert!(parse_set("cut").is_none());
        assert!(matches!(
            parse_set("a="),
            Some(SetCommand::Var { name, value }) if name == "a" && value.is_empty()
        ));
        assert!(parse_set("1a=x").is_none());
        assert!(matches!(parse_set("a+=1"), Some(SetCommand::Step(name)) if name == "a"));
        assert!(matches!(parse_set("a += 1"), Some(SetCommand::Step(name)) if name == "a"));
        assert!(parse_set("a+=2").is_none());
        assert!(parse_set("1a+=1").is_none());
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
        assert!(!settings.set_var("copy", "no".into()));
        assert!(!settings.set_var("home", "no".into()));
        assert!(!settings.set_var("cut", "no".into()));
        assert_eq!(
            Settings::load(path.clone()).vars().get("a").map(String::as_str),
            Some("{{date}}")
        );
        assert!(settings.set_var("a", String::new()));
        assert!(Settings::load(path.clone()).vars().get("a").is_none());
        fs::write(
            &path,
            r#"{"version":1,"shortcuts":{"register":"Control+Digit4","show":"Control+Digit7"},"vars":{"ok":"1","paste":"x","copy":"y","1bad":"z"}}"#,
        )
        .unwrap();
        assert!(settings.reload());
        assert_eq!(settings.vars().get("ok").map(String::as_str), Some("1"));
        assert!(!settings.vars().contains_key("paste"));
        assert!(!settings.vars().contains_key("copy"));
        assert!(settings.vars().get("1bad").is_none());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn step_starts_at_one_and_bumps_after_paste() {
        let path = temp_path("steps");
        let _ = fs::remove_file(&path);
        let mut settings = Settings::load(path.clone());
        assert!(settings.arm_step("a"));
        assert_eq!(settings.vars().get("a").map(String::as_str), Some("1"));
        assert_eq!(settings.format_vars(), "a=1 +1");
        settings.bump_steps(&["a".into(), "missing".into()]);
        assert_eq!(
            Settings::load(path.clone()).vars().get("a").map(String::as_str),
            Some("2")
        );
        assert!(settings.set_var("a", "10".into()));
        settings.bump_steps(&["a".into()]);
        assert_eq!(settings.vars().get("a").map(String::as_str), Some("11"));
        assert!(settings.set_var("b", "{{date}}".into()));
        assert!(!settings.arm_step("b"));
        assert!(settings.set_var("a", String::new()));
        assert!(settings.vars().get("a").is_none());
        assert_eq!(settings.format_vars(), "b={{date}}");
        let _ = fs::remove_file(path);
    }
}
