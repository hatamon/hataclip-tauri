// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    match decide(std::env::args().skip(1), stdin_is_redirected()) {
        Launch::Gui => hataclip_lib::run(),
        Launch::Cli { expr, redirected } => {
            std::process::exit(hataclip_lib::run_cli(&expr, redirected))
        }
        Launch::Fail => std::process::exit(1),
    }
}

enum Launch {
    Gui,
    Cli { expr: String, redirected: bool },
    Fail,
}

/// 引数が無ければトレイに常駐する。式が1つならそれを実行する。2つ以上は失敗。
fn decide(args: impl IntoIterator<Item = String>, redirected: bool) -> Launch {
    let mut args = args.into_iter();
    let expr = args.next();
    if args.next().is_some() {
        return Launch::Fail;
    }
    match expr {
        None if !redirected => Launch::Gui,
        Some(expr) => Launch::Cli { expr, redirected },
        None => Launch::Fail,
    }
}

/// 標準入力がパイプかファイルなら真。コンソールと、ハンドルの無い起動は偽。
///
/// リリースの Windows 版はコンソールを持たない。`is_terminal` はそこでも偽になるので、
/// ダブルクリックや PowerShell からの起動までパイプ扱いにして即終了していた。
fn stdin_is_redirected() -> bool {
    #[cfg(windows)]
    {
        windows_stdin_is_redirected()
    }
    #[cfg(not(windows))]
    {
        !std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}

#[cfg(windows)]
fn windows_stdin_is_redirected() -> bool {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Storage::FileSystem::GetFileType;
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_INPUT_HANDLE};

    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return false;
    }
    stdin_kind_is_redirected(unsafe { GetFileType(handle) })
}

/// `GetFileType` の戻り。`FILE_TYPE_REMOTE` は種類と OR されるので外す。
#[cfg(any(windows, test))]
fn stdin_kind_is_redirected(kind: u32) -> bool {
    const FILE_TYPE_DISK: u32 = 0x0001;
    const FILE_TYPE_PIPE: u32 = 0x0003;
    const FILE_TYPE_REMOTE: u32 = 0x8000;
    let kind = kind & !FILE_TYPE_REMOTE;
    kind == FILE_TYPE_DISK || kind == FILE_TYPE_PIPE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_without_redirect_starts_the_tray() {
        assert!(matches!(decide(None, false), Launch::Gui));
    }

    #[test]
    fn no_args_with_redirect_exits() {
        assert!(matches!(decide(None, true), Launch::Fail));
    }

    #[test]
    fn one_expr_runs_cli() {
        match decide(Some("echo 1".to_string()), false) {
            Launch::Cli { expr, redirected } => {
                assert_eq!(expr, "echo 1");
                assert!(!redirected);
            }
            _ => panic!("cli"),
        }
        match decide(Some("quote".to_string()), true) {
            Launch::Cli { expr, redirected } => {
                assert_eq!(expr, "quote");
                assert!(redirected);
            }
            _ => panic!("cli"),
        }
    }

    #[test]
    fn two_args_fail() {
        let args = ["echo 1".to_string(), "quote".to_string()];
        assert!(matches!(decide(args, false), Launch::Fail));
    }

    #[test]
    fn only_pipe_and_file_count_as_redirected() {
        const FILE_TYPE_UNKNOWN: u32 = 0x0000;
        const FILE_TYPE_DISK: u32 = 0x0001;
        const FILE_TYPE_CHAR: u32 = 0x0002;
        const FILE_TYPE_PIPE: u32 = 0x0003;
        const FILE_TYPE_REMOTE: u32 = 0x8000;
        assert!(!stdin_kind_is_redirected(FILE_TYPE_UNKNOWN));
        assert!(stdin_kind_is_redirected(FILE_TYPE_DISK));
        assert!(!stdin_kind_is_redirected(FILE_TYPE_CHAR));
        assert!(stdin_kind_is_redirected(FILE_TYPE_PIPE));
        assert!(stdin_kind_is_redirected(FILE_TYPE_PIPE | FILE_TYPE_REMOTE));
        assert!(!stdin_kind_is_redirected(FILE_TYPE_CHAR | FILE_TYPE_REMOTE));
    }
}
