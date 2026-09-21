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
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            register: DEFAULT_REGISTER.to_string(),
            show: DEFAULT_SHOW.to_string(),
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

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    shortcuts: Shortcuts,
    #[serde(default)]
    window: Option<WindowGeom>,
}

pub struct Settings {
    path: PathBuf,
    shortcuts: Shortcuts,
    window: Option<WindowGeom>,
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let (shortcuts, window) = read_file(&path);
        Self {
            path,
            shortcuts,
            window,
        }
    }

    pub fn shortcuts(&self) -> &Shortcuts {
        &self.shortcuts
    }

    pub fn window(&self) -> Option<WindowGeom> {
        self.window
    }

    pub fn set_shortcuts(&mut self, shortcuts: Shortcuts) {
        self.shortcuts = shortcuts;
        self.save();
    }

    pub fn set_window(&mut self, window: WindowGeom) {
        self.window = Some(window);
        self.save();
    }

    fn save(&self) {
        let _ = write_file(&self.path, &self.shortcuts, self.window);
    }
}

fn read_file(path: &Path) -> (Shortcuts, Option<WindowGeom>) {
    let Ok(data) = fs::read_to_string(path) else {
        return (Shortcuts::default(), None);
    };
    let Ok(file) = serde_json::from_str::<SettingsFile>(&data) else {
        return (Shortcuts::default(), None);
    };
    let shortcuts = Shortcuts {
        register: usable(file.shortcuts.register, DEFAULT_REGISTER),
        show: usable(file.shortcuts.show, DEFAULT_SHOW),
    };
    (shortcuts, file.window)
}

fn usable(value: String, fallback: &str) -> String {
    if actions::parse(&value).is_ok() {
        value
    } else {
        fallback.to_string()
    }
}

fn write_file(
    path: &Path,
    shortcuts: &Shortcuts,
    window: Option<WindowGeom>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = SettingsFile {
        version: CURRENT_VERSION,
        shortcuts: shortcuts.clone(),
        window,
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
}
