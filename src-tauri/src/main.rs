fn main() {
    #[cfg(windows)]
    std::process::exit(windows_cli());

    #[cfg(not(windows))]
    match decide(std::env::args().skip(1), stdin_is_redirected()) {
        Launch::Gui => hataclip_lib::run(),
        Launch::Cli { expr, redirected } => {
            let stdin = if redirected {
                Some(read_piped_stdin())
            } else {
                None
            };
            std::process::exit(hataclip_lib::run_cli(&expr, stdin.as_deref()))
        }
        Launch::Fail => std::process::exit(1),
    }
}

/// Windows のパイプ用はコンソール付きの `hataclip.exe`。トレイは `hataclip-gui.exe`。
#[cfg(windows)]
fn windows_cli() -> i32 {
    let mut args = std::env::args().skip(1);
    let Some(expr) = args.next() else {
        eprintln!("式を1つ渡す");
        return 1;
    };
    if args.next().is_some() {
        eprintln!("式は1つ");
        return 1;
    }
    let stdin = if stdin_is_redirected() {
        Some(read_piped_stdin())
    } else {
        None
    };
    hataclip_lib::run_cli(&expr, stdin.as_deref())
}

/// 引数が無ければトレイに常駐する。式が1つならそれを実行する。2つ以上は失敗。
#[cfg(not(windows))]
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

#[cfg(not(windows))]
enum Launch {
    Gui,
    Cli { expr: String, redirected: bool },
    Fail,
}

/// 標準入力がパイプかファイルなら真。
fn stdin_is_redirected() -> bool {
    !std::io::IsTerminal::is_terminal(&std::io::stdin())
}

fn read_piped_stdin() -> String {
    use std::io::Read;
    let mut buf = Vec::new();
    if std::io::stdin().read_to_end(&mut buf).is_err() {
        std::process::exit(1);
    }
    String::from_utf8_lossy(&buf).into_owned()
}

#[cfg(all(test, not(windows)))]
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
}
