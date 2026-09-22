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
        Parse::Bad if braced.len() < 2 => PasteBody::Text,
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
        if only.is_empty() || peel_clip(only).1 {
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
    let (peeled, clip) = peel_clip(&body[body.len() - 1]);
    if clip {
        sink = "clip".to_string();
        if peeled.is_empty() {
            body.pop();
        } else {
            body.pop();
            body.push(peeled);
        }
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
    if middle_clip(&ops) {
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

fn quote_prefix(line: &str) -> Option<String> {
    if line == "quote" {
        return Some("> ".to_string());
    }
    if let Some(rest) = line.strip_prefix("quote ").or_else(|| line.strip_prefix("quote\t")) {
        return Some(colon_arg(rest, "> "));
    }
    None
}

fn join_separator(line: &str) -> Option<String> {
    if line == "comma" || line == "join" {
        return Some(",".to_string());
    }
    if line == "tab" {
        return Some("\t".to_string());
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
    if part == "raw" {
        return Some(op("raw", "", false));
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
    if let Some(prefix) = quote_prefix(part) {
        return Some(op("quote", &prefix, false));
    }
    if let Some(sep) = join_separator(part) {
        return Some(op("join", &sep, false));
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
}
