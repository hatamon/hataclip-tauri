/// 履歴の本文がパイプかどうか。`{{ }}` の中の `|` だけではパイプにしない。

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Op {
    pub kind: String,
    pub arg: String,
    pub selection_stdin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Script {
    pub ops: Vec<Op>,
    pub sink: String,
    pub set_name: Option<String>,
    pub uses_selection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PasteBody {
    Text,
    Bad,
    Run(Script),
}

pub(crate) fn classify(text: &str) -> PasteBody {
    let line = text.trim();
    let braced = split_bars(line, true);
    match parse(line) {
        Parse::None => PasteBody::Text,
        Parse::Bad if braced.len() < 2 && !peel_clip(line).1 => PasteBody::Text,
        Parse::Bad => PasteBody::Bad,
        Parse::Ok(script) => PasteBody::Run(script),
    }
}

enum Parse {
    None,
    Bad,
    Ok(Script),
}

fn parse(input: &str) -> Parse {
    let line = input.trim();
    let line = line.strip_prefix(':').unwrap_or(line);
    let parts = split_bars(line, false);
    if parts.len() < 2 {
        let only = parts.first().map(String::as_str).unwrap_or("");
        if only.is_empty() {
            return Parse::None;
        }
        if peel_clip(only).1 {
            return Parse::Bad;
        }
        if only == "add" {
            return Parse::None;
        }
        let Some(stage) = stage_of(only) else {
            return Parse::None;
        };
        let uses_selection = pipe_uses_selection(std::slice::from_ref(&stage));
        return Parse::Ok(Script {
            ops: vec![stage],
            sink: "paste".to_string(),
            set_name: None,
            uses_selection,
        });
    }
    if parts.iter().any(|part| part.is_empty()) {
        return Parse::Bad;
    }
    let mut sink = "paste".to_string();
    let mut set_name = None;
    let mut body = parts;
    let (_peeled, clip) = peel_clip(&body[body.len() - 1]);
    if clip {
        return Parse::Bad;
    } else if let Some((name, var)) = sink_of(&body[body.len() - 1]) {
        sink = name;
        set_name = var;
        body.pop();
    }
    if body.is_empty() {
        return Parse::Bad;
    }
    let mut ops = Vec::new();
    for part in &body {
        let Some(stage) = stage_of(part) else {
            return Parse::Bad;
        };
        ops.push(stage);
    }
    if sh_after_each(&ops) {
        return Parse::Bad;
    }
    let uses_selection = pipe_uses_selection(&ops);
    Parse::Ok(Script {
        ops,
        sink,
        set_name,
        uses_selection,
    })
}

fn split_bars(line: &str, braces: bool) -> Vec<String> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut depth = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if escaped {
            buf.push(ch);
            escaped = false;
            index += 1;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            buf.push(ch);
            escaped = true;
            index += 1;
            continue;
        }
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
            buf.push(ch);
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            buf.push(ch);
            index += 1;
            continue;
        }
        if braces && ch == '{' && chars.get(index + 1) == Some(&'{') {
            depth += 1;
            buf.push_str("{{");
            index += 2;
            continue;
        }
        if braces && ch == '}' && chars.get(index + 1) == Some(&'}') && depth > 0 {
            depth -= 1;
            buf.push_str("}}");
            index += 2;
            continue;
        }
        if ch == '|' && depth == 0 {
            parts.push(buf.trim().to_string());
            buf.clear();
            index += 1;
            continue;
        }
        buf.push(ch);
        index += 1;
    }
    parts.push(buf.trim().to_string());
    parts
}

fn unquote(raw: &str) -> Option<String> {
    let text = raw.trim_start();
    let mut chars = text.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest: Vec<char> = chars.collect();
    let mut out = String::new();
    let mut escaped = false;
    let mut index = 0;
    while index < rest.len() {
        let ch = rest[index];
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
            let tail: String = rest[index..].iter().collect();
            if !tail.trim().is_empty() {
                return None;
            }
            return Some(out);
        }
        out.push(ch);
    }
    None
}

fn colon_arg(rest: &str, fallback: &str) -> String {
    if let Some(quoted) = unquote(rest) {
        return quoted;
    }
    let plain = rest.trim();
    if plain.is_empty() {
        fallback.to_string()
    } else {
        plain.to_string()
    }
}

fn read_quoted(raw: &str) -> Option<(String, &str)> {
    let text = raw.trim_start();
    let mut chars = text.char_indices();
    let (quote_at, quote) = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let _ = quote_at;
    let body: Vec<char> = chars.map(|(_, ch)| ch).collect();
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
            let skipped = text.len() - tail.len();
            return Some((out, &text[skipped..]));
        }
        out.push(ch);
    }
    None
}

fn quote_marks(line: &str) -> Option<(String, String)> {
    if line == "quote" {
        return Some(("> ".to_string(), String::new()));
    }
    let rest = line
        .strip_prefix("quote ")
        .or_else(|| line.strip_prefix("quote\t"))?;
    if rest.trim().is_empty() {
        return Some(("> ".to_string(), String::new()));
    }
    if let Some((value, tail)) = read_quoted(rest) {
        if tail.trim().is_empty() {
            return Some((value, String::new()));
        }
        let (suffix, tail) = read_quoted(tail)?;
        if !tail.trim().is_empty() {
            return None;
        }
        return Some((value, suffix));
    }
    Some((colon_arg(rest, "> "), String::new()))
}

fn encode_quote(prefix: &str, suffix: &str) -> String {
    if suffix.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}\u{1}{suffix}")
    }
}

fn filter_arg(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("filter ")
        .or_else(|| line.strip_prefix("filter\t"))?;
    let rest = rest.trim();
    if rest.is_empty() || rest == "not" {
        return None;
    }
    let (invert, raw) = if let Some(after) = rest
        .strip_prefix("not ")
        .or_else(|| rest.strip_prefix("not\t"))
    {
        let after = after.trim();
        if after.is_empty() {
            return None;
        }
        (true, after)
    } else {
        (false, rest)
    };
    let needle = if let Some(quoted) = unquote(raw) {
        if quoted.is_empty() {
            return None;
        }
        quoted
    } else if raw.starts_with('"') || raw.starts_with('\'') {
        return None;
    } else {
        raw.to_string()
    };
    if invert {
        Some(format!("\u{1}{needle}"))
    } else {
        Some(needle)
    }
}

fn split_separator(line: &str) -> Option<String> {
    let rest = line.strip_prefix("split ").or_else(|| line.strip_prefix("split\t"))?;
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }
    if let Some(quoted) = unquote(rest) {
        if quoted.is_empty() {
            return None;
        }
        return Some(quoted);
    }
    if rest.starts_with('"') || rest.starts_with('\'') {
        return None;
    }
    Some(rest.to_string())
}

fn col_arg(line: &str) -> Option<String> {
    let rest = line.strip_prefix("col ").or_else(|| line.strip_prefix("col\t"))?;
    let rest = rest.trim();
    if rest.is_empty() || !rest.chars().all(|ch| ch.is_ascii_digit() || ch == '-') {
        return None;
    }
    Some(rest.to_string())
}

fn put_arg(line: &str) -> Option<String> {
    let rest = line.strip_prefix("put ").or_else(|| line.strip_prefix("put\t"))?;
    let rest = rest.trim_start();
    let (pointer, raw) = rest.split_once(char::is_whitespace)?;
    let raw = raw.trim();
    if !pointer.starts_with('/') || raw.is_empty() {
        return None;
    }
    let value = if let Some(quoted) = unquote(raw) {
        quoted
    } else if raw.starts_with('"') || raw.starts_with('\'') {
        return None;
    } else {
        raw.to_string()
    };
    Some(format!("{pointer}\u{1}{value}"))
}

fn side_arg(line: &str, name: &str) -> Option<String> {
    let rest = line
        .strip_prefix(&format!("{name} "))
        .or_else(|| line.strip_prefix(&format!("{name}\t")))?;
    let rest = rest.trim();
    if rest == "." || rest == "clip" {
        Some(rest.to_string())
    } else {
        None
    }
}

fn pointer_arg(line: &str, name: &str) -> Option<String> {
    let rest = line
        .strip_prefix(&format!("{name} "))
        .or_else(|| line.strip_prefix(&format!("{name}\t")))?;
    let rest = rest.trim();
    if !rest.starts_with('/') {
        return None;
    }
    Some(rest.to_string())
}

fn join_separator(line: &str) -> Option<String> {
    if line == "join" {
        return Some(",".to_string());
    }
    if let Some(rest) = line.strip_prefix("join ").or_else(|| line.strip_prefix("join\t")) {
        return Some(colon_arg(rest, ","));
    }
    None
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn var_sink_name(part: &str) -> Option<String> {
    let rest = part.strip_prefix("set ").or_else(|| part.strip_prefix("set\t"))?;
    let name = rest.trim();
    if name == "paste" || !is_ident(name) {
        return None;
    }
    Some(name.to_string())
}

fn sink_of(part: &str) -> Option<(String, Option<String>)> {
    if part == "clip" || part == "add" || part == "open" || part == "show" {
        return Some((part.to_string(), None));
    }
    if let Some(path) = log_sink(part) {
        return Some(("log".to_string(), path));
    }
    var_sink_name(part).map(|name| ("set".to_string(), Some(name)))
}

/// `| log` か `| log path`。パスが無いときは `None`（変数を使う）。
fn log_sink(part: &str) -> Option<Option<String>> {
    if part == "log" {
        return Some(None);
    }
    let rest = part.strip_prefix("log ").or_else(|| part.strip_prefix("log\t"))?;
    let path = rest.trim();
    if path.is_empty() {
        return Some(None);
    }
    Some(Some(path.to_string()))
}

/// 段のない行き先。標準入力があるときだけ使う。`clip` だけの履歴行はクリップボードを読む段のまま。
pub(crate) fn stdin_sink(expr: &str) -> Option<Script> {
    let line = expr.trim();
    let line = line.strip_prefix(':').unwrap_or(line);
    if split_bars(line, false).len() != 1 {
        return None;
    }
    let (sink, set_name) = sink_of(line)?;
    Some(Script {
        ops: Vec::new(),
        sink,
        set_name,
        uses_selection: false,
    })
}

/// `s/old/new`。`old` が空なら段にしない。`new` は2つ目の `/` の後ろ全部。
fn sub_arg(part: &str) -> Option<String> {
    let rest = part.strip_prefix("s/")?;
    let (old, new) = rest.split_once('/')?;
    if old.is_empty() {
        return None;
    }
    Some(format!("{old}\u{1}{new}"))
}

fn sh_script(raw: &str) -> String {
    if let Some(quoted) = unquote(raw) {
        return quoted.trim().to_string();
    }
    raw.trim().to_string()
}

fn stage_of(part: &str) -> Option<Op> {
    if part == "." {
        return Some(op("dot", "", false));
    }
    if part == "clip" {
        return Some(op("clip", "", false));
    }
    if part == "add" {
        return Some(op("add", "", false));
    }
    if part == "show" {
        return Some(op("show", "", false));
    }
    if part == "sel" {
        return Some(op("sel", "", false));
    }
    if part == "raw" || part == "each" {
        return Some(op(part, "", false));
    }
    if part == "format" {
        return Some(op("format", "", false));
    }
    if matches!(
        part,
        "camel" | "pascal" | "snake" | "kebab" | "upper" | "lower" | "json" | "xml"
    ) {
        return Some(op(part, "", false));
    }
    if part == "echo" || part.starts_with("echo ") || part.starts_with("echo\t") {
        let expr = if part == "echo" { "" } else { part[4..].trim() };
        if expr.is_empty() {
            return None;
        }
        return Some(op("echo", expr, false));
    }
    if let Some((prefix, suffix)) = quote_marks(part) {
        return Some(op("quote", &encode_quote(&prefix, &suffix), false));
    }
    if let Some(sep) = join_separator(part) {
        return Some(op("join", &sep, false));
    }
    if let Some(sep) = split_separator(part) {
        return Some(op("split", &sep, false));
    }
    if let Some(arg) = filter_arg(part) {
        return Some(op("filter", &arg, false));
    }
    if let Some(index) = col_arg(part) {
        return Some(op("col", &index, false));
    }
    if let Some(pointer) = pointer_arg(part, "get") {
        return Some(op("get", &pointer, false));
    }
    if let Some(side) = side_arg(part, "diff") {
        return Some(op("diff", &side, false));
    }
    if let Some(side) = side_arg(part, "only") {
        return Some(op("only", &side, false));
    }
    if let Some(arg) = put_arg(part) {
        return Some(op("put", &arg, false));
    }
    if let Some(arg) = sub_arg(part) {
        return Some(op("sub", &arg, false));
    }
    if part == "sh" || part.starts_with("sh ") || part.starts_with("sh\t") {
        let raw = if part == "sh" { "" } else { &part[2..] };
        let script = sh_script(raw.trim_start());
        if script.is_empty() {
            return None;
        }
        return Some(op("sh", &script, false));
    }
    if let Some(raw) = dot_sh_raw(part) {
        let script = sh_script(raw);
        if script.is_empty() {
            return None;
        }
        return Some(op("sh", &script, true));
    }
    None
}

fn dot_sh_raw(part: &str) -> Option<&str> {
    if let Some(rest) = part.strip_prefix(".! sh ").or_else(|| part.strip_prefix(".! sh\t")) {
        return Some(rest);
    }
    if let Some(rest) = part.strip_prefix(".!sh ").or_else(|| part.strip_prefix(".!sh\t")) {
        return Some(rest);
    }
    None
}

fn op(kind: &str, arg: &str, selection_stdin: bool) -> Op {
    Op {
        kind: kind.to_string(),
        arg: arg.to_string(),
        selection_stdin,
    }
}

fn peel_clip(part: &str) -> (String, bool) {
    let lower = part.to_ascii_lowercase();
    let Some(index) = lower.rfind('>') else {
        return (part.to_string(), false);
    };
    if lower[index + 1..].trim() != "clip" {
        return (part.to_string(), false);
    }
    (part[..index].trim().to_string(), true)
}

fn sh_after_each(ops: &[Op]) -> bool {
    let mut seen = false;
    for op in ops {
        if op.kind == "each" {
            seen = true;
        }
        if seen && op.kind == "sh" {
            return true;
        }
    }
    false
}

/// `:` の入力中に出す結果。`sh` と `sel` は出さない。12行を超えたら末尾に `...`。
pub(crate) fn colon_preview(body: &str, dot: &str, clip: &str) -> Option<String> {
    let line = body.trim().trim_start_matches(':').trim();
    if line.is_empty() {
        return None;
    }
    let PasteBody::Run(script) = classify(line) else {
        return None;
    };
    if script.ops.len() == 1 && script.ops[0].kind == "sub" && script.sink == "paste" {
        return None;
    }
    if script.ops.iter().any(|op| op.kind == "sh" || op.kind == "sel") {
        return None;
    }
    let text = eval_local(&script.ops, None, "", dot, clip)?;
    if text.is_empty() {
        return None;
    }
    Some(clip_preview(&text, 12))
}

fn clip_preview(text: &str, limit: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= limit {
        return text.to_string();
    }
    let mut out = lines[..limit].join("\n");
    out.push_str("\n...");
    out
}

fn eval_local(
    ops: &[Op],
    mut text: Option<String>,
    sel: &str,
    dot: &str,
    clip: &str,
) -> Option<String> {
    let mut index = 0;
    while index < ops.len() {
        let op = &ops[index];
        if op.kind == "each" {
            let body = text.take()?;
            let mut kept = Vec::new();
            for line in body.lines() {
                if let Some(out) =
                    eval_local(&ops[index + 1..], Some(line.to_string()), sel, dot, clip)
                {
                    kept.push(out);
                }
            }
            if kept.is_empty() {
                return None;
            }
            return Some(kept.join("\n"));
        }
        match op.kind.as_str() {
            "raw" => {}
            "sel" => {
                if text.is_none() {
                    if sel.is_empty() {
                        return None;
                    }
                    text = Some(sel.to_string());
                }
            }
            "clip" => {
                if text.is_none() {
                    text = Some(clip.to_string());
                }
            }
            "add" | "show" => {
                if text.is_none() {
                    return None;
                }
            }
            "dot" => {
                if text.is_none() {
                    if dot.is_empty() {
                        return None;
                    }
                    text = Some(dot.to_string());
                }
            }
            "echo" => {
                let value = crate::expr::eval_with(&op.arg, &std::collections::HashMap::new())?;
                text = Some(crate::expr::format_number(value));
            }
            "sh" => return None,
            "json" | "xml" => {
                let current = take_or_dot(&mut text, dot)?;
                let pretty = if op.kind == "json" {
                    crate::text::pretty_json(&current)
                } else {
                    crate::text::pretty_xml(&current)
                };
                text = Some(pretty?);
            }
            "put" => {
                let current = take_or_dot(&mut text, dot)?;
                let (pointer, value) = op.arg.split_once('\u{1}')?;
                text = Some(crate::text::json_put(&current, pointer, value)?);
            }
            "diff" | "only" => {
                let current = take_or_dot(&mut text, dot)?;
                let other = match op.arg.as_str() {
                    "clip" => clip,
                    _ => return None,
                };
                let next = if op.kind == "diff" {
                    crate::text::line_diff(&current, other)
                } else {
                    crate::text::only_lines(&current, other)
                };
                text = Some(next?);
            }
            "filter" => {
                let current = take_or_dot(&mut text, dot)?;
                text = Some(crate::text::filter_lines(&current, &op.arg)?);
            }
            "split" | "col" | "get" => {
                let current = take_or_dot(&mut text, dot)?;
                let next = match op.kind.as_str() {
                    "split" => Some(crate::text::split_fields(&current, &op.arg)),
                    "col" => {
                        let index = op.arg.parse::<i64>().unwrap_or(0);
                        crate::text::take_column(&current, index)
                    }
                    "get" => crate::text::json_at(&current, &op.arg),
                    _ => None,
                };
                text = Some(next?);
            }
            "quote" | "format" | "join" | "sub" | "camel" | "pascal" | "snake" | "kebab" | "upper"
            | "lower" => {
                let current = take_or_dot(&mut text, dot)?;
                text = Some(match op.kind.as_str() {
                    "join" => current.lines().collect::<Vec<_>>().join(&op.arg),
                    "quote" => {
                        let (prefix, suffix) = crate::text::split_quote_arg(&op.arg);
                        crate::text::affix_lines(&current, prefix, suffix)
                    }
                    "format" => crate::text::format_for_paste(&current),
                    "sub" => crate::text::substitute_literal(&current, &op.arg),
                    "camel" | "pascal" | "snake" | "kebab" | "upper" | "lower" => {
                        crate::text::recase(&current, &op.kind)
                    }
                    _ => current,
                });
            }
            _ => return None,
        }
        index += 1;
    }
    text
}

fn take_or_dot(text: &mut Option<String>, dot: &str) -> Option<String> {
    if text.is_none() && !dot.is_empty() {
        *text = Some(dot.to_string());
    }
    text.take()
}

pub(crate) fn render_ops(ops: &[Op]) -> String {
    ops.iter().map(render_op).collect::<Vec<_>>().join(" | ")
}

fn render_op(op: &Op) -> String {
    match op.kind.as_str() {
        "dot" => ".".to_string(),
        "clip" | "sel" | "raw" | "each" | "format" | "json" | "xml" | "camel" | "pascal"
        | "snake" | "kebab" | "upper" | "lower" | "add" | "show" => op.kind.clone(),
        "echo" => format!("echo {}", op.arg),
        "sh" => {
            let script = quote_render(&op.arg);
            if op.selection_stdin {
                format!(".!sh {script}")
            } else {
                format!("sh {script}")
            }
        }
        "quote" => {
            let (prefix, suffix) = crate::text::split_quote_arg(&op.arg);
            if suffix.is_empty() {
                format!("quote {}", quote_render(&prefix))
            } else {
                format!("quote {} {}", quote_render(&prefix), quote_render(&suffix))
            }
        }
        "join" => format!("join {}", quote_render(&op.arg)),
        "split" => format!("split {}", quote_render(&op.arg)),
        "filter" => {
            let (invert, needle) = if let Some(needle) = op.arg.strip_prefix('\u{1}') {
                (true, needle)
            } else {
                (false, op.arg.as_str())
            };
            if invert {
                format!("filter not {}", quote_render(needle))
            } else {
                format!("filter {}", quote_render(needle))
            }
        }
        "col" => format!("col {}", op.arg),
        "get" => format!("get {}", op.arg),
        "diff" => format!("diff {}", op.arg),
        "only" => format!("only {}", op.arg),
        "put" => {
            let (pointer, value) = op.arg.split_once('\u{1}').unwrap_or((op.arg.as_str(), ""));
            format!("put {pointer} {}", quote_render(value))
        }
        "sub" => {
            let (old, new) = op.arg.split_once('\u{1}').unwrap_or((op.arg.as_str(), ""));
            format!("s/{old}/{new}")
        }
        other => other.to_string(),
    }
}

fn quote_render(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn pipe_uses_selection(ops: &[Op]) -> bool {
    let mut produced = false;
    for op in ops {
        if op.kind == "raw" {
            continue;
        }
        if op.kind == "clip" || op.kind == "sel" {
            produced = true;
            continue;
        }
        if op.kind == "dot" {
            if !produced {
                return true;
            }
            produced = true;
            continue;
        }
        if op.kind == "sh" {
            if !produced && op.selection_stdin {
                return true;
            }
            produced = true;
            continue;
        }
        if op.kind == "echo" {
            produced = true;
            continue;
        }
        if !produced {
            return true;
        }
    }
    !produced
}

/// Ctrl+Shift+H の複数行。1行目が `:` のパイプなら、残りがその流れ。
pub(crate) enum Headed {
    /// 今までの展開（`{{date}}` や1行の `:echo` / `:sh`）に戻す。
    Skip,
    /// この形だが実行しない。
    Noop,
    Run { script: Script, flow: String },
}

pub(crate) fn headed_pipe(text: &str) -> Headed {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let Some((first, rest)) = normalized.split_once('\n') else {
        return Headed::Skip;
    };
    let first = first.trim();
    if let Some(path) = log_sink(first.strip_prefix(':').unwrap_or(first)) {
        let flow = rest.trim_end_matches('\n').to_string();
        if flow.is_empty() {
            return Headed::Noop;
        }
        return Headed::Run {
            script: Script {
                ops: vec![op("raw", "", false)],
                sink: "log".to_string(),
                set_name: path,
                uses_selection: false,
            },
            flow,
        };
    }
    if !first.starts_with(':') {
        return Headed::Skip;
    }
    let flow = rest.trim_end_matches('\n').to_string();
    match classify(first) {
        PasteBody::Text => Headed::Skip,
        PasteBody::Bad => Headed::Noop,
        PasteBody::Run(script) if flow.is_empty() && self_contained(&script) => Headed::Skip,
        PasteBody::Run(script) if plain_echo(&script) => Headed::Noop,
        PasteBody::Run(_) if flow.is_empty() => Headed::Noop,
        PasteBody::Run(script) => Headed::Run { script, flow },
    }
}

fn plain_echo(script: &Script) -> bool {
    script.sink == "paste" && script.ops.len() == 1 && script.ops[0].kind == "echo"
}

/// `sh` か `echo` が流れを作り、`.` `sel` `clip` は読まない。
pub(crate) fn self_contained(script: &Script) -> bool {
    if script.ops.iter().any(|op| {
        matches!(op.kind.as_str(), "sel" | "clip" | "dot")
            || (op.kind == "sh" && op.selection_stdin)
            || (matches!(op.kind.as_str(), "diff" | "only") && op.arg == ".")
    }) {
        return false;
    }
    let mut produced = false;
    for op in &script.ops {
        if op.kind == "raw" || op.kind == "each" {
            continue;
        }
        if op.kind == "echo" || op.kind == "sh" {
            produced = true;
            continue;
        }
        if !produced {
            return false;
        }
    }
    produced
}

pub(crate) enum Solo {
    Skip,
    Noop,
    Run(Script),
}

/// 1行（末尾の改行だけは空）の Ctrl+Shift+H。複数行の本体があるときは `Skip`。
pub(crate) fn solo_pipe(text: &str) -> Solo {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let line = match normalized.split_once('\n') {
        Some((first, rest)) if rest.trim().is_empty() => first.trim(),
        Some(_) => return Solo::Skip,
        None => normalized.trim(),
    };
    if line.is_empty() {
        return Solo::Skip;
    }
    match classify(line) {
        PasteBody::Run(script) if self_contained(&script) => Solo::Run(script),
        PasteBody::Bad => Solo::Noop,
        _ => Solo::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_stage_runs_as_a_pipe() {
        let PasteBody::Run(script) = classify(":upper") else {
            panic!("run");
        };
        assert_eq!(script.sink, "paste");
        assert!(script.uses_selection);
        assert_eq!(script.ops[0].kind, "upper");
        let PasteBody::Run(echo) = classify("echo 3+4") else {
            panic!("echo");
        };
        assert!(!echo.uses_selection);
        assert_eq!(echo.ops[0].arg, "3+4");
        assert_eq!(classify("hello"), PasteBody::Text);
        assert_eq!(classify("add"), PasteBody::Text);
        let bare = stdin_sink("add").expect("add");
        assert_eq!(bare.sink, "add");
        assert!(bare.ops.is_empty());
        assert!(stdin_sink("quote | add").is_none());
        assert!(stdin_sink("upper").is_none());
        assert_eq!(classify("help"), PasteBody::Text);
        assert_eq!(classify("comma"), PasteBody::Text);
        let PasteBody::Run(split) = classify(r#"sel | split "\t" | col 2 | get /name"#) else {
            panic!("split");
        };
        assert_eq!(split.ops[1].kind, "split");
        assert_eq!(split.ops[1].arg, "\t");
        assert_eq!(split.ops[2].arg, "2");
        assert_eq!(split.ops[3].arg, "/name");
        assert!(matches!(classify("sel | each | get /id"), PasteBody::Run(_)));
        assert_eq!(classify("sel | each | sh dir"), PasteBody::Bad);
        assert_eq!(classify("sel | split"), PasteBody::Bad);
        assert_eq!(classify("sel | col x"), PasteBody::Bad);
        assert_eq!(classify("sel | comma"), PasteBody::Bad);
        assert_eq!(classify("sel | tab"), PasteBody::Bad);
        assert_eq!(classify("quote \"* \" > clip"), PasteBody::Bad);
        assert_eq!(classify("sh dir>clip"), PasteBody::Bad);
        let PasteBody::Run(quoted) = classify(r#"quote "> " "<""#) else {
            panic!("quote");
        };
        assert_eq!(quoted.ops[0].arg, format!("> \u{1}<"));
    }

    #[test]
    fn template_bar_inside_braces_stays_text() {
        assert_eq!(classify("{{sel|clip}}"), PasteBody::Text);
        assert_eq!(classify("{{when a|b}}x{{when}}y"), PasteBody::Text);
        assert_eq!(classify("hello"), PasteBody::Text);
    }

    #[test]
    fn sel_pipe_runs_without_the_list_selection() {
        let PasteBody::Run(script) = classify(":sel | upper") else {
            panic!("run");
        };
        assert_eq!(script.sink, "paste");
        assert!(!script.uses_selection);
        assert_eq!(script.ops[0].kind, "sel");
        assert_eq!(script.ops[1].kind, "upper");
    }

    #[test]
    fn unknown_stage_is_rejected_and_middle_clip_passes_through() {
        assert_eq!(classify("sel | snak"), PasteBody::Bad);
        let PasteBody::Run(script) = classify("sel | clip | upper") else {
            panic!("run");
        };
        assert_eq!(script.sink, "paste");
        assert_eq!(
            script.ops.iter().map(|op| op.kind.as_str()).collect::<Vec<_>>(),
            vec!["sel", "clip", "upper"]
        );
        let PasteBody::Run(shown) = classify("echo 1 | clip | show") else {
            panic!("show");
        };
        assert_eq!(shown.sink, "show");
        assert_eq!(colon_preview(". | clip | upper", "ab", "ZZ").as_deref(), Some("AB"));
        assert_eq!(classify("add"), PasteBody::Text);
        let PasteBody::Run(added) = classify("kebab | add | quote") else {
            panic!("add");
        };
        assert_eq!(
            added.ops.iter().map(|op| op.kind.as_str()).collect::<Vec<_>>(),
            vec!["kebab", "add", "quote"]
        );
        assert_eq!(
            colon_preview("kebab | add | quote", "userName", "").as_deref(),
            Some("> user-name")
        );
        let PasteBody::Run(shown) = classify("show | add") else {
            panic!("show");
        };
        assert_eq!(shown.sink, "add");
        assert_eq!(shown.ops[0].kind, "show");
    }

    #[test]
    fn quote_into_clip_is_a_clip_sink() {
        let PasteBody::Run(script) = classify(":sel | quote \"> \" | clip") else {
            panic!("run");
        };
        assert_eq!(script.sink, "clip");
        assert_eq!(script.ops.len(), 2);
        assert_eq!(script.ops[1].kind, "quote");
        assert_eq!(script.ops[1].arg, "> ");
    }

    #[test]
    fn quoted_bar_stays_inside_sh() {
        let PasteBody::Run(script) = classify("sh \"dir | sort\" | show") else {
            panic!("run");
        };
        assert_eq!(script.sink, "show");
        assert_eq!(script.ops[0].kind, "sh");
        assert_eq!(script.ops[0].arg, "dir | sort");
        let PasteBody::Run(echo) = classify("echo 3+4 | show") else {
            panic!("echo");
        };
        assert_eq!(echo.ops[0].arg, "3+4");
    }

    #[test]
    fn dot_uses_the_list_row() {
        let PasteBody::Run(script) = classify(". | upper") else {
            panic!("run");
        };
        assert!(script.uses_selection);
        assert_eq!(script.sink, "paste");
    }

    #[test]
    fn colon_preview_skips_sel_and_sh() {
        assert!(colon_preview(":sel | kebab", "", "").is_none());
        assert_eq!(colon_preview("upper", "ab", "").as_deref(), Some("AB"));
        assert!(colon_preview("sel | sh dir", "", "").is_none());
        assert!(colon_preview("sel | json", "", "").is_none());
        assert!(colon_preview("nope", "", "").is_none());
        assert!(colon_preview("s/old/new", "", "").is_none());
        assert!(colon_preview("sel | s/old/new", "", "").is_none());
        let long = "ab\n".repeat(20);
        let shown = colon_preview("upper", &long, "").unwrap();
        assert!(shown.ends_with("\n..."));
        assert_eq!(shown.lines().count(), 13);
    }

    #[test]
    fn rendered_formula_parses_again() {
        let PasteBody::Run(script) = classify("sel | kebab") else {
            panic!("run");
        };
        assert_eq!(render_ops(&script.ops), "sel | kebab");
        let PasteBody::Run(again) = classify(&render_ops(&script.ops)) else {
            panic!("again");
        };
        assert_eq!(again.ops[0].kind, "sel");
        assert_eq!(again.ops[1].kind, "kebab");
    }

    #[test]
    fn substitute_replaces_every_occurrence() {
        let PasteBody::Run(script) = classify(":s/old/new") else {
            panic!("run");
        };
        assert_eq!(script.ops[0].kind, "sub");
        assert_eq!(
            eval_local(
                &script.ops,
                Some("abc def old ghi\noldabc ddd".into()),
                "",
                "",
                ""
            )
            .as_deref(),
            Some("abc def new ghi\nnewabc ddd")
        );
        assert_eq!(
            eval_local(&script.ops, Some("none".into()), "", "", "").as_deref(),
            Some("none")
        );
        assert!(matches!(classify(":s//new"), PasteBody::Text));
        assert!(matches!(classify("s/old/new | upper"), PasteBody::Run(_)));
        let PasteBody::Run(quoted) = classify(":quote | upper") else {
            panic!("quote");
        };
        assert_eq!(
            eval_local(&quoted.ops, Some("xxx\nyyy".into()), "", "", "").as_deref(),
            Some("> XXX\n> YYY")
        );
    }

    #[test]
    fn headed_pipe_runs_the_first_line_on_the_rest() {
        match headed_pipe(":s/old/new\nabc def old ghi\noldabc ddd") {
            Headed::Run { script, flow } => {
                assert_eq!(script.ops[0].kind, "sub");
                assert_eq!(flow, "abc def old ghi\noldabc ddd");
            }
            _ => panic!("run"),
        }
        match headed_pipe(":quote | upper\nxxx\nyyy") {
            Headed::Run { flow, .. } => assert_eq!(flow, "xxx\nyyy"),
            _ => panic!("quote"),
        }
        assert!(matches!(headed_pipe(":echo 2+3"), Headed::Skip));
        assert!(matches!(headed_pipe(":echo 2+3\nnotes"), Headed::Noop));
        assert!(matches!(headed_pipe(":sh dir | quote\n"), Headed::Skip));
        assert!(matches!(headed_pipe(":quote\n"), Headed::Noop));
        assert!(matches!(headed_pipe(":sh dir"), Headed::Skip));
        assert!(matches!(headed_pipe("hello\nworld"), Headed::Skip));
        match headed_pipe("log\nabc\ndef") {
            Headed::Run { script, flow } => {
                assert_eq!(script.sink, "log");
                assert!(script.set_name.is_none());
                assert_eq!(flow, "abc\ndef");
            }
            _ => panic!("log"),
        }
        match headed_pipe(":log notes.log\nabc") {
            Headed::Run { script, flow } => {
                assert_eq!(script.set_name.as_deref(), Some("notes.log"));
                assert_eq!(flow, "abc");
            }
            _ => panic!("path"),
        }
        assert!(matches!(headed_pipe("log\n"), Headed::Noop));
        let PasteBody::Run(logged) = classify("echo 3+4|log") else {
            panic!("echo log");
        };
        assert_eq!(logged.sink, "log");
        assert_eq!(logged.ops[0].kind, "echo");
        assert!(matches!(headed_pipe(":s/old/new"), Headed::Skip));
        assert!(matches!(headed_pipe(":s/old/new\n"), Headed::Noop));
        assert!(matches!(headed_pipe(":nope | zz\nx"), Headed::Noop));
        let PasteBody::Run(script) = classify(":sel | s/old/new") else {
            panic!("sel");
        };
        assert_eq!(
            eval_local(&script.ops, Some("a old".into()), "other", "", "").as_deref(),
            Some("a new")
        );
    }

    #[test]
    fn solo_pipe_runs_a_line_that_makes_its_own_flow() {
        match solo_pipe(r#":sh "dir | sort" | quote ">" "<""#) {
            Solo::Run(script) => {
                assert_eq!(script.ops[0].kind, "sh");
                assert!(!script.ops[0].selection_stdin);
                assert_eq!(script.ops[0].arg, "dir | sort");
                assert_eq!(script.ops[1].kind, "quote");
                assert_eq!(script.ops[1].arg, format!(">\u{1}<"));
            }
            _ => panic!("quoted shell pipe"),
        }
        match solo_pipe(":echo 2+3 | quote") {
            Solo::Run(script) => {
                assert_eq!(
                    eval_local(&script.ops, None, "", "", "").as_deref(),
                    Some("> 5")
                );
            }
            _ => panic!("echo"),
        }
        match solo_pipe(":echo 2+3 | quote \">\" \"<\"\n") {
            Solo::Run(script) => {
                assert_eq!(
                    eval_local(&script.ops, None, "", "", "").as_deref(),
                    Some(">5<")
                );
            }
            _ => panic!("marks"),
        }
        assert!(matches!(solo_pipe(":sh dir | sort"), Solo::Noop));
        assert!(matches!(solo_pipe(":quote"), Solo::Skip));
        assert!(matches!(solo_pipe(":sel | upper"), Solo::Skip));
        assert!(matches!(solo_pipe(":s/old/new"), Solo::Skip));
        assert!(matches!(solo_pipe("{{date}}"), Solo::Skip));
        assert!(matches!(solo_pipe(":sh dir | quote\nfoo"), Solo::Skip));
        assert!(matches!(solo_pipe(":clip | upper"), Solo::Skip));
        assert!(matches!(solo_pipe(":. | upper"), Solo::Skip));
    }

    #[test]
    fn filter_keeps_or_drops_lines_that_contain_the_needle() {
        let PasteBody::Run(script) = classify(r#":sel | filter ".txt""#) else {
            panic!("keep");
        };
        assert_eq!(script.ops[1].kind, "filter");
        assert_eq!(script.ops[1].arg, ".txt");
        assert_eq!(
            eval_local(&script.ops, None, "a.txt\nb.rs\nc.TXT", "", "").as_deref(),
            Some("a.txt")
        );
        let PasteBody::Run(dropped) = classify(r#"filter not ".txt""#) else {
            panic!("drop");
        };
        assert_eq!(dropped.ops[0].arg, "\u{1}.txt");
        assert_eq!(
            eval_local(&dropped.ops, Some("a.txt\nb.rs".into()), "", "", "").as_deref(),
            Some("b.rs")
        );
        assert!(eval_local(&script.ops, None, "b.rs", "", "").is_none());
        assert!(matches!(classify("filter"), PasteBody::Text));
        assert!(matches!(classify("filter not"), PasteBody::Text));
        assert!(matches!(classify(r#"sel | filter """#), PasteBody::Bad));
        let PasteBody::Run(word) = classify(r#"filter "not""#) else {
            panic!("word");
        };
        assert_eq!(word.ops[0].arg, "not");
        let PasteBody::Run(inverted) = classify(r#"filter not "not""#) else {
            panic!("inverted");
        };
        assert_eq!(inverted.ops[0].arg, "\u{1}not");
        assert_eq!(
            render_ops(&dropped.ops),
            r#"filter not ".txt""#
        );
        let PasteBody::Run(again) = classify(&render_ops(&dropped.ops)) else {
            panic!("again");
        };
        assert_eq!(again.ops[0].arg, dropped.ops[0].arg);
        assert!(matches!(solo_pipe(r#":filter ".txt""#), Solo::Skip));
        match solo_pipe(r#":echo 1 | filter "1""#) {
            Solo::Run(script) => {
                assert_eq!(eval_local(&script.ops, None, "", "", "").as_deref(), Some("1"));
            }
            _ => panic!("solo"),
        }
    }
}
