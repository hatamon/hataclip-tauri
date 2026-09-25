use crate::pipe::{self, Op};
use crate::settings::Settings;
use crate::store::{Item, Store};
use crate::text;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn run(expr: &str, stdin: Option<&str>, show_error: bool) -> i32 {
    match execute(expr, stdin) {
        Ok(CliResult::Stdout(text)) => write_stdout(&text),
        Ok(CliResult::Show(text)) => match crate::page::present(&text) {
            Ok(()) => 0,
            Err(_) => 1,
        },
        Ok(CliResult::Quiet) => 0,
        Err(message) => {
            if show_error && !message.trim().is_empty() {
                eprintln!("{message}");
            }
            1
        }
    }
}

fn write_stdout(text: &str) -> i32 {
    let mut out = io::stdout().lock();
    if out.write_all(text.as_bytes()).is_err() {
        return 1;
    }
    if !text.ends_with('\n') && out.write_all(b"\n").is_err() {
        return 1;
    }
    0
}

enum CliResult {
    Stdout(String),
    Show(String),
    Quiet,
}

fn resolve(expr: &str, piped: bool) -> Result<pipe::Script, String> {
    if piped {
        if let Some(script) = pipe::stdin_sink(expr) {
            return Ok(script);
        }
    }
    match pipe::classify(expr) {
        pipe::PasteBody::Run(script) => Ok(script),
        pipe::PasteBody::Text | pipe::PasteBody::Bad => Err("段が読めない".into()),
    }
}

fn execute(expr: &str, stdin: Option<&str>) -> Result<CliResult, String> {
    let script = resolve(expr, stdin.is_some())?;
    if script.ops.iter().any(|op| op.kind == "sel") {
        return Err("選択を取れない".into());
    }
    let dir = data_dir().ok_or_else(|| "失敗".to_string())?;
    let mut store = Store::load(dir.join("items.json"));
    let mut settings = Settings::load(dir.join("settings.json"));
    let vars: std::collections::HashMap<String, String> = settings
        .vars()
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();
    let mut added = Vec::new();
    let text = walk(
        &script.ops,
        stdin.map(str::to_string),
        &vars,
        stdin.is_some(),
        &mut added,
    )?;
    for (body, formula) in added {
        let tags = text::auto_tags(&body);
        let mut item = Item::new(body, tags);
        item.formula = formula;
        store.insert(item);
    }
    match script.sink.as_str() {
        "paste" => {
            if text.is_empty() {
                return Err("空".into());
            }
            Ok(CliResult::Stdout(text))
        }
        "show" => {
            if text.is_empty() {
                return Err("空".into());
            }
            Ok(CliResult::Show(text))
        }
        "clip" => {
            if text.is_empty() {
                return Err("空".into());
            }
            if !crate::clipboard::write_clipboard_text(&text) {
                return Err("クリップボードに書けない".into());
            }
            Ok(CliResult::Quiet)
        }
        "add" => {
            if text.is_empty() {
                return Err("空".into());
            }
            let tags = text::auto_tags(&text);
            let mut item = Item::new(text, tags);
            item.formula = pipe::render_ops(&script.ops);
            store.insert(item);
            Ok(CliResult::Quiet)
        }
        "set" => {
            let name = script.set_name.as_deref().unwrap_or("");
            if !settings.set_var(name, text) {
                return Err("名前が違う".into());
            }
            Ok(CliResult::Quiet)
        }
        "open" => {
            let target = text.trim();
            if target.starts_with("http://")
                || target.starts_with("https://")
                || text::looks_like_path(target)
            {
                crate::platform::open_target(target);
            }
            Ok(CliResult::Quiet)
        }
        "log" => {
            if text.is_empty() {
                return Err("失敗".to_string());
            }
            let vars = settings.vars();
            let Some(path) = crate::log_destination(
                script.set_name.as_deref().unwrap_or(""),
                vars.get("defaultLogFileName").map(String::as_str),
            ) else {
                return Ok(CliResult::Quiet);
            };
            if crate::append_log(&path, &text).is_err() {
                return Err("書けない".into());
            }
            Ok(CliResult::Quiet)
        }
        _ => Err("段が読めない".into()),
    }
}

fn walk(
    ops: &[Op],
    seed: Option<String>,
    vars: &std::collections::HashMap<String, String>,
    piped: bool,
    added: &mut Vec<(String, String)>,
) -> Result<String, String> {
    let mut text = seed;
    let mut index = 0;
    while index < ops.len() {
        let op = &ops[index];
        if op.kind == "each" {
            let body = take_text(&mut text, piped)?;
            let mut kept = Vec::new();
            for line in body.lines() {
                if let Ok(out) = walk(&ops[index + 1..], Some(line.to_string()), vars, true, added) {
                    kept.push(out);
                }
            }
            if kept.is_empty() {
                return Err("失敗".to_string());
            }
            return Ok(kept.join("\n"));
        }
        match op.kind.as_str() {
            "raw" | "dot" => {
                if text.is_none() {
                    if op.kind == "dot" && !piped {
                        return Err("失敗".to_string());
                    }
                    text = Some(String::new());
                }
            }
            "add" => {
                let Some(current) = text.as_deref() else {
                    return Err("失敗".to_string());
                };
                if !current.is_empty() {
                    added.push((current.to_string(), pipe::render_ops(&ops[..index])));
                }
            }
            "clip" => match text.as_deref() {
                Some(current) => {
                    if !current.is_empty() && !crate::clipboard::write_clipboard_text(current) {
                        return Err("失敗".to_string());
                    }
                }
                None => {
                    text = Some(crate::clipboard::peek_text().unwrap_or_default());
                }
            },
            "echo" => {
                let value = crate::expr::eval_with(&op.arg, vars).ok_or_else(|| "失敗".to_string())?;
                text = Some(crate::expr::format_number(value));
            }
            "sh" => {
                let stdin = if op.selection_stdin || text.is_some() {
                    Some(take_text(&mut text, piped)?)
                } else {
                    None
                };
                let output = crate::shell::run_script_with_stdin(&op.arg, stdin.as_deref())
                    .map_err(|err| crate::shell::command_error(&err))?;
                text = Some(output);
            }
            "json" | "xml" => {
                let current = take_text(&mut text, piped)?;
                let pretty = if op.kind == "json" {
                    text::pretty_json(&current)
                } else {
                    text::pretty_xml(&current)
                };
                text = Some(pretty.ok_or_else(|| "失敗".to_string())?);
            }
            "put" => {
                let current = take_text(&mut text, piped)?;
                let (pointer, value) = op.arg.split_once('\u{1}').ok_or_else(|| "失敗".to_string())?;
                text = Some(text::json_put(&current, pointer, value).ok_or_else(|| "失敗".to_string())?);
            }
            "diff" | "only" => {
                let current = take_text(&mut text, piped)?;
                let other = match op.arg.as_str() {
                    "clip" => crate::clipboard::peek_text().unwrap_or_default(),
                    "." => return Err("失敗".to_string()),
                    _ => return Err("失敗".to_string()),
                };
                let next = if op.kind == "diff" {
                    text::line_diff(&current, &other)
                } else {
                    text::only_lines(&current, &other)
                };
                text = Some(next.ok_or_else(|| "失敗".to_string())?);
            }
            "filter" => {
                let current = take_text(&mut text, piped)?;
                text = Some(text::filter_lines(&current, &op.arg).ok_or_else(|| "失敗".to_string())?);
            }
            "split" | "col" | "get" => {
                let current = take_text(&mut text, piped)?;
                let next = match op.kind.as_str() {
                    "split" => Some(text::split_fields(&current, &op.arg)),
                    "col" => {
                        let index = op.arg.parse::<i64>().unwrap_or(0);
                        text::take_column(&current, index)
                    }
                    "get" => text::json_at(&current, &op.arg),
                    _ => None,
                };
                text = Some(next.ok_or_else(|| "失敗".to_string())?);
            }
            "quote" | "format" | "join" | "sub" | "camel" | "pascal" | "snake" | "kebab" | "upper"
            | "lower" => {
                if text.is_none() && !piped {
                    return Err("失敗".to_string());
                }
                let current = text.take().unwrap_or_default();
                text = Some(match op.kind.as_str() {
                    "join" => current.lines().collect::<Vec<_>>().join(&op.arg),
                    "quote" => {
                        let (prefix, suffix) = text::split_quote_arg(&op.arg);
                        text::affix_lines(&current, prefix, suffix)
                    }
                    "format" => text::format_for_paste(&current),
                    "camel" | "pascal" | "snake" | "kebab" | "upper" | "lower" => {
                        text::recase(&current, &op.kind)
                    }
                    "sub" => text::substitute_literal(&current, &op.arg),
                    _ => current,
                });
            }
            _ => return Err("失敗".to_string()),
        }
        index += 1;
    }
    text.ok_or_else(|| "空".to_string())
}

fn take_text(text: &mut Option<String>, piped: bool) -> Result<String, String> {
    if text.is_none() && !piped {
        return Err("空".into());
    }
    Ok(text.take().unwrap_or_default())
}

#[cfg(test)]
fn transform(expr: &str, stdin: Option<&str>) -> Result<String, String> {
    let script = resolve(expr, stdin.is_some())?;
    if script.ops.iter().any(|op| op.kind == "sel") {
        return Err("選択を取れない".into());
    }
    let mut added = Vec::new();
    walk(
        &script.ops,
        stdin.map(str::to_string),
        &std::collections::HashMap::new(),
        stdin.is_some(),
        &mut added,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_and_quote_and_columns() {
        assert_eq!(transform("echo 3+4", None).as_deref(), Ok("7"));
        assert_eq!(transform("quote", Some("hello")).as_deref(), Ok("> hello"));
        assert_eq!(transform("split , | col 2", Some("a,b,c")).as_deref(), Ok("b"));
        assert_eq!(transform("upper", Some("Ab")).as_deref(), Ok("AB"));
        assert_eq!(
            transform("s/old/new", Some("abc old")).as_deref(),
            Ok("abc new")
        );
        assert!(transform("json", Some("{")).is_err());
        assert!(transform("sel | upper", Some("hello")).is_err());
        assert!(transform("nope", Some("hello")).is_err());
        assert!(transform("quote", None).is_err());
        assert_eq!(transform("add", Some("hello")).as_deref(), Ok("hello"));
        assert_eq!(transform("clip", Some("hello")).as_deref(), Ok("hello"));
        assert_eq!(transform("show", Some("hello")).as_deref(), Ok("hello"));
        assert!(transform("add", None).is_err());
        let script = resolve("kebab | add | quote", true).expect("pipe");
        let mut added = Vec::new();
        let text = walk(
            &script.ops,
            Some("userName".into()),
            &std::collections::HashMap::new(),
            true,
            &mut added,
        )
        .expect("walk");
        assert_eq!(text, "> user-name");
        assert_eq!(added, vec![("user-name".into(), "kebab".into())]);
        let bare = resolve("add | upper", false).expect("add");
        let mut added = Vec::new();
        assert!(walk(
            &bare.ops,
            None,
            &std::collections::HashMap::new(),
            false,
            &mut added,
        )
        .is_err());
    }
}

fn data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let base = std::env::var_os("APPDATA")?;
        Some(PathBuf::from(base).join("com.hataclip.app"))
    }
    #[cfg(not(windows))]
    {
        if let Some(base) = std::env::var_os("XDG_DATA_HOME") {
            return Some(PathBuf::from(base).join("com.hataclip.app"));
        }
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".local/share/com.hataclip.app"))
    }
}
