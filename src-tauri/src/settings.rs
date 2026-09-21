use crate::actions;
use serde::{Deserialize, Serialize};
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
}

fn bool_true() -> bool {
    true
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            register: DEFAULT_REGISTER.to_string(),
            show: DEFAULT_SHOW.to_string(),
            quick_paste: true,
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
    "\\".to_string()
}

impl Default for KeyMaps {
    fn default() -> Self {
        Self {
            leader: default_leader(),
            maps: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    shortcuts: Shortcuts,
    #[serde(default)]
    window: Option<WindowGeom>,
    #[serde(default)]
    keymaps: KeyMaps,
}

pub struct Settings {
    path: PathBuf,
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
    keymaps: KeyMaps,
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let (shortcuts, window, keymaps) = read_file(&path);
        Self {
            path,
            shortcuts,
            window,
            keymaps,
        }
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
        let _ = write_file(&self.path, &self.shortcuts, self.window, &self.keymaps);
    }
}

fn read_file(path: &Path) -> (Shortcuts, Option<WindowGeom>, KeyMaps) {
    let Ok(data) = fs::read_to_string(path) else {
        return (Shortcuts::default(), None, KeyMaps::default());
    };
    let Ok(file) = serde_json::from_str::<SettingsFile>(&data) else {
        return (Shortcuts::default(), None, KeyMaps::default());
    };
    let shortcuts = Shortcuts {
        register: usable(file.shortcuts.register, DEFAULT_REGISTER),
        show: usable(file.shortcuts.show, DEFAULT_SHOW),
        quick_paste: file.shortcuts.quick_paste,
    };
    (shortcuts, file.window, sanitize_keymaps(file.keymaps))
}

fn usable(value: String, fallback: &str) -> String {
    if actions::parse(&value).is_ok() {
        value
    } else {
        fallback.to_string()
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

fn write_file(
    path: &Path,
    shortcuts: &Shortcuts,
    window: Option<WindowGeom>,
    keymaps: &KeyMaps,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = SettingsFile {
        version: CURRENT_VERSION,
        shortcuts: shortcuts.clone(),
        window,
        keymaps: keymaps.clone(),
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
        assert_eq!(
            reloaded.window(),
            Some(WindowGeom {
                x: 10,
                y: 20,
                width: 400,
                height: 300,
            })
        );
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
}
