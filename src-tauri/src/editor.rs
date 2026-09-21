use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// nvim を優先し、無ければ $VISUAL / $EDITOR、それも無ければメモ帳。
pub fn find() -> Option<PathBuf> {
    if let Some(path) = lookup("nvim") {
        return Some(path);
    }
    for key in ["VISUAL", "EDITOR"] {
        let Ok(value) = std::env::var(key) else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        if value.is_empty() {
            continue;
        }
        if let Some(path) = lookup(value) {
            return Some(path);
        }
    }
    if cfg!(windows) {
        return lookup("notepad");
    }
    None
}

fn lookup(program: &str) -> Option<PathBuf> {
    let direct = Path::new(program);
    if direct.components().count() > 1 {
        return direct.is_file().then(|| direct.to_path_buf());
    }
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let base = dir.join(program);
        if base.is_file() {
            return Some(base);
        }
        for extension in executable_extensions() {
            let candidate = dir.join(format!("{program}{extension}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn executable_extensions() -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }
    std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string())
        .split(';')
        .filter(|extension| !extension.is_empty())
        .map(|extension| extension.to_lowercase())
        .collect()
}

pub fn write_temp_file(id: &str, text: &str) -> io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("hataclip-{id}.txt"));
    std::fs::write(&path, text)?;
    Ok(path)
}

/// エディタが閉じるまで待ち、書き換わった本文を返す。変わっていなければ None。
pub fn run(editor: &Path, file: &Path) -> Option<String> {
    let before = std::fs::read_to_string(file).ok();
    let mut command = Command::new(editor);
    command.arg(file);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // コンソールを持たない GUI アプリなので、エディタには新しいコンソールを与える。
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        command.creation_flags(CREATE_NEW_CONSOLE);
    }
    let finished = command
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    let after = std::fs::read_to_string(file).ok();
    let _ = std::fs::remove_file(file);
    if !finished {
        return None;
    }
    let after = after?;
    if Some(&after) == before.as_ref() {
        return None;
    }
    Some(normalize(&after))
}

/// エディタが閉じるまで待つ。ファイルは消さない。
pub fn wait_close(editor: &Path, file: &Path) -> bool {
    let mut command = Command::new(editor);
    command.arg(file);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        command.creation_flags(CREATE_NEW_CONSOLE);
    }
    command
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn normalize(text: &str) -> String {
    let text = text.replace("\r\n", "\n");
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_the_trailing_newline_an_editor_adds() {
        assert_eq!(normalize("hello\n"), "hello");
        assert_eq!(normalize("a\r\nb\r\n"), "a\nb");
        assert_eq!(normalize("hello"), "hello");
    }

    #[test]
    fn keeps_blank_lines_in_the_middle() {
        assert_eq!(normalize("a\n\nb\n"), "a\n\nb");
    }
}
