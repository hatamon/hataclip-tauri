use std::path::Path;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
mod input;
#[cfg(windows)]
pub use windows::{
    app_and_title, capture_foreground, context_key, restore_foreground, Foreground,
};
#[cfg(windows)]
pub use input::{simulate_copy, simulate_paste, simulate_type, simulate_type_atoms};

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::{
    app_and_title, capture_foreground, context_key, restore_foreground, Foreground, open_dir,
    open_file, open_url,
};

#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// フォーカスしている入力欄の種類。Ubuntu と未実装時は空。
pub fn focused_control() -> String {
    #[cfg(windows)]
    {
        windows::focused_control()
    }
    #[cfg(not(windows))]
    {
        String::new()
    }
}

/// Linux は前面へ送れないのでクリップボードに残す。Windows は注入して戻す。
#[cfg_attr(not(test), allow(dead_code))]
pub fn keeps_clipboard_on_paste() -> bool {
    cfg!(not(windows))
}

pub fn open_target(text: &str) -> bool {
    let text = text.trim();
    if text.starts_with("http://") || text.starts_with("https://") {
        return open_url(text);
    }
    if !crate::text::looks_like_path(text) {
        return false;
    }
    let path = Path::new(text);
    if path.is_file() {
        return open_file(text);
    }
    if path.is_dir() {
        return open_dir(text);
    }
    false
}

#[cfg(windows)]
pub fn open_url(url: &str) -> bool {
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .is_ok()
}

#[cfg(windows)]
pub fn open_file(text: &str) -> bool {
    Command::new("explorer")
        .arg(format!("/select,{text}"))
        .spawn()
        .is_ok()
}

#[cfg(windows)]
pub fn open_dir(text: &str) -> bool {
    Command::new("explorer").arg(text).spawn().is_ok()
}

#[cfg(test)]
mod tests {
    use super::keeps_clipboard_on_paste;

    #[test]
    fn clipboard_policy_matches_os() {
        assert_eq!(keeps_clipboard_on_paste(), !cfg!(windows));
    }
}
