fn main() {
    let redirected = stdin_is_redirected();
    #[cfg(windows)]
    let idle_is_tray = false;
    #[cfg(not(windows))]
    let idle_is_tray = true;
    match hataclip_lib::parse_command(std::env::args().skip(1), redirected, idle_is_tray) {
        hataclip_lib::Command::Gui => hataclip_lib::run(),
        hataclip_lib::Command::Help(topic) => {
            let text = hataclip_lib::render_help(topic.as_deref());
            std::process::exit(hataclip_lib::present_text(&text));
        }
        hataclip_lib::Command::Run { expr, show_error } => {
            let stdin = if redirected {
                Some(read_piped_stdin())
            } else {
                None
            };
            std::process::exit(hataclip_lib::run_cli(&expr, stdin.as_deref(), show_error));
        }
        hataclip_lib::Command::Fail { message, show_error } => {
            if hataclip_lib::fail_text(message, show_error, cfg!(windows)).is_some() {
                eprintln!("{message}");
            }
            std::process::exit(1);
        }
    }
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
