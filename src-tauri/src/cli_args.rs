#[derive(Debug)]
pub enum Command {
    Gui,
    Help(Option<String>),
    Run { expr: String, show_error: bool },
    Fail { message: &'static str, show_error: bool },
}

/// `--help` と `--showerror` は式ではない。式は1つだけ。
pub fn parse_command(
    args: impl IntoIterator<Item = impl AsRef<str>>,
    redirected: bool,
    idle_is_tray: bool,
) -> Command {
    let mut help = false;
    let mut show_error = false;
    let mut positionals = Vec::new();
    for arg in args {
        match arg.as_ref() {
            "--help" => help = true,
            "--showerror" => show_error = true,
            other => positionals.push(other.to_string()),
        }
    }
    if help {
        return match positionals.len() {
            0 => Command::Help(None),
            1 => Command::Help(Some(positionals.remove(0))),
            _ => Command::Fail {
                message: "式は1つ",
                show_error,
            },
        };
    }
    match positionals.len() {
        0 if idle_is_tray && !redirected && !show_error => Command::Gui,
        0 => Command::Fail {
            message: "式を1つ渡す",
            show_error,
        },
        1 => Command::Run {
            expr: positionals.remove(0),
            show_error,
        },
        _ => Command::Fail {
            message: "式は1つ",
            show_error,
        },
    }
}

pub fn fail_text(message: &str, show_error: bool, always: bool) -> Option<&str> {
    if show_error || always {
        Some(message)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_not_an_expression() {
        assert!(matches!(
            parse_command(["--help"], false, true),
            Command::Help(None)
        ));
        match parse_command(["--help", "quote"], false, false) {
            Command::Help(Some(topic)) => assert_eq!(topic, "quote"),
            other => panic!("{other:?}"),
        }
        match parse_command(["quote", "--help"], true, true) {
            Command::Help(Some(topic)) => assert_eq!(topic, "quote"),
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            parse_command(["--help", "quote", "extra"], false, true),
            Command::Fail { message: "式は1つ", .. }
        ));
    }

    #[test]
    fn showerror_keeps_one_expression_and_stdin() {
        match parse_command(["--showerror", "echo nope"], true, true) {
            Command::Run { expr, show_error } => {
                assert_eq!(expr, "echo nope");
                assert!(show_error);
            }
            other => panic!("{other:?}"),
        }
        match parse_command(["echo nope", "--showerror"], false, false) {
            Command::Run { expr, show_error } => {
                assert_eq!(expr, "echo nope");
                assert!(show_error);
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            parse_command(["--showerror"], false, true),
            Command::Fail {
                message: "式を1つ渡す",
                show_error: true,
            }
        ));
        assert!(matches!(
            parse_command(["--showerror", "echo 1", "echo 2"], true, true),
            Command::Fail {
                message: "式は1つ",
                show_error: true,
            }
        ));
        assert!(matches!(
            parse_command(None::<String>, false, true),
            Command::Gui
        ));
        assert!(matches!(
            parse_command(None::<String>, true, true),
            Command::Fail { show_error: false, .. }
        ));
        assert!(matches!(
            parse_command(None::<String>, false, false),
            Command::Fail {
                message: "式を1つ渡す",
                show_error: false,
            }
        ));
    }

    #[test]
    fn fail_text_is_only_for_the_flag_unless_windows_always_prints() {
        assert_eq!(fail_text("空", false, false), None);
        assert_eq!(fail_text("空", true, false), Some("空"));
        assert_eq!(fail_text("式は1つ", false, true), Some("式は1つ"));
    }
}
