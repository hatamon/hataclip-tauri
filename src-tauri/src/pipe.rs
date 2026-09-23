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
    if middle_clip(&ops) || sh_after_each(&ops) {
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
    var_sink_name(part).map(|name| ("set".to_string(), Some(name)))
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

fn middle_clip(ops: &[Op]) -> bool {
    let mut produced = false;
    for op in ops {
        if op.kind == "clip" {
            if produced {
                return true;
            }
            produced = true;
            continue;
        }
        if op.kind == "raw" {
            continue;
        }
        produced = true;
    }
    false
}

/// 一覧を開いたときのプレビュー。`sel` を前面の選択にし、`sh` は実行しない。
pub(crate) fn selection_preview(body: &str, sel: &str, clip: &str) -> Option<String> {
    let PasteBody::Run(script) = classify(body) else {
        return None;
    };
    if script.ops.iter().any(|op| op.kind == "sh") {
        return None;
    }
    if !script.ops.iter().any(|op| op.kind == "sel") {
        return None;
    }
    eval_local(&script.ops, None, sel, clip).filter(|text| !text.is_empty())
}

fn eval_local(ops: &[Op], mut text: Option<String>, sel: &str, clip: &str) -> Option<String> {
    let mut index = 0;
    while index < ops.len() {
        let op = &ops[index];
        if op.kind == "each" {
            let body = text.take()?;
            let mut kept = Vec::new();
            for line in body.lines() {
                if let Some(out) = eval_local(&ops[index + 1..], Some(line.to_string()), sel, clip)
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
                if text.is_some() {
                    return None;
                }
                text = Some(clip.to_string());
            }
            "dot" => return None,
            "echo" => {
                let value = crate::expr::eval_with(&op.arg, &std::collections::HashMap::new())?;
                text = Some(crate::expr::format_number(value));
            }
            "sh" => return None,
            "json" | "xml" => {
                let current = text.take()?;
                let pretty = if op.kind == "json" {
                    crate::text::pretty_json(&current)
                } else {
                    crate::text::pretty_xml(&current)
                };
                text = Some(pretty?);
            }
            "put" => {
                let current = text.take()?;
                let (pointer, value) = op.arg.split_once('\u{1}')?;
                text = Some(crate::text::json_put(&current, pointer, value)?);
            }
            "diff" | "only" => {
                let current = text.take()?;
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
            "split" | "col" | "get" => {
                let current = text.take()?;
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
            "quote" | "format" | "join" | "camel" | "pascal" | "snake" | "kebab" | "upper"
            | "lower" => {
                let current = text.take()?;
                text = Some(match op.kind.as_str() {
                    "join" => current.lines().collect::<Vec<_>>().join(&op.arg),
                    "quote" => {
                        let (prefix, suffix) = crate::text::split_quote_arg(&op.arg);
                        crate::text::affix_lines(&current, prefix, suffix)
                    }
                    "format" => crate::text::format_for_paste(&current),
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
    fn unknown_stage_and_middle_clip_do_nothing() {
        assert_eq!(classify("sel | snak"), PasteBody::Bad);
        assert_eq!(classify("sel | clip | upper"), PasteBody::Bad);
        assert_eq!(classify("echo 1 | clip | show"), PasteBody::Bad);
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
    fn selection_preview_skips_sh_and_rows_without_sel() {
        assert_eq!(
            selection_preview(":sel | upper", "hello", "").as_deref(),
            Some("HELLO")
        );
        assert_eq!(
            selection_preview("sel | split , | col 2", "a,b,c", "").as_deref(),
            Some("b")
        );
        assert!(selection_preview("sel | sh dir", "hello", "").is_none());
        assert!(selection_preview("upper", "hello", "").is_none());
        assert!(selection_preview("sel | json", "{", "").is_none());
        assert!(selection_preview("hello", "hello", "").is_none());
    }
}
