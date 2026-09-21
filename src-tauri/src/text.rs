/// 登録した本文から自動で付けるタグ。
pub fn auto_tags(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    let mut tags = Vec::new();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        tags.push("url".to_string());
    }
    if !tags.iter().any(|tag| tag == "url") && looks_like_path(trimmed) {
        tags.push("path".to_string());
    }
    tags
}

pub fn looks_like_path(text: &str) -> bool {
    if text.contains('\n') || text.chars().count() > 260 {
        return false;
    }
    let bytes = text.as_bytes();
    let drive = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/');
    drive || text.starts_with("\\\\") || text.starts_with("~/") || text.starts_with('/')
}

/// 貼り付け直前のテンプレート展開。履歴の本文自体は書き換えない。
pub struct Expand {
    pub date: String,
    pub time: String,
    pub clip: String,
    pub sel: String,
    pub n: u32,
    pub uuid: String,
    pub user: String,
    pub host: String,
    pub app: String,
    pub front: String,
    pub now: chrono::DateTime<chrono::Local>,
    pub answers: std::collections::HashMap<String, String>,
    pub aliases: std::collections::HashMap<String, String>,
    pub vars: std::collections::HashMap<String, String>,
    pub tags: std::collections::HashMap<String, String>,
}

/// 貼り付けの断片。`{{type:}}` のところだけキー、ほかはテキスト。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasteOp {
    Text(String),
    Type(Vec<crate::keys::TypeAtom>),
    Wait(u64),
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn expand_ops(text: &str, ctx: &Expand) -> Vec<PasteOp> {
    take_type_ops(&expand_template(text, ctx))
}

/// 展開済みの本文から `{{type:}}` だけ切り出す。
pub fn take_type_ops(text: &str) -> Vec<PasteOp> {
    compact_ops(take_type_ops_raw(text))
}

fn take_type_ops_raw(text: &str) -> Vec<PasteOp> {
    let chars: Vec<char> = text.chars().collect();
    let mut ops = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                let token = inner.trim();
                if let Some(script) = arg_after(token, "type") {
                    if !buf.is_empty() {
                        ops.push(PasteOp::Text(std::mem::take(&mut buf)));
                    }
                    ops.push(PasteOp::Type(crate::keys::parse_type_script(script)));
                    i = close + 2;
                    continue;
                }
                if let Some(ms) = arg_after(token, "wait") {
                    if let Some(wait) = parse_wait(ms) {
                        if !buf.is_empty() {
                            ops.push(PasteOp::Text(std::mem::take(&mut buf)));
                        }
                        ops.push(PasteOp::Wait(wait));
                        i = close + 2;
                        continue;
                    }
                }
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        ops.push(PasteOp::Text(buf));
    }
    ops
}

pub fn compact_ops(ops: Vec<PasteOp>) -> Vec<PasteOp> {
    let mut out = Vec::new();
    for op in ops {
        match op {
            PasteOp::Text(text) if text.is_empty() => {}
            PasteOp::Type(atoms) if atoms.is_empty() => {}
            PasteOp::Wait(0) => {}
            PasteOp::Text(text) => {
                if let Some(PasteOp::Text(last)) = out.last_mut() {
                    last.push_str(&text);
                } else {
                    out.push(PasteOp::Text(text));
                }
            }
            PasteOp::Type(atoms) => {
                if let Some(PasteOp::Type(last)) = out.last_mut() {
                    last.extend(atoms);
                } else {
                    out.push(PasteOp::Type(atoms));
                }
            }
            other => out.push(other),
        }
    }
    out
}

pub fn flatten_ops(ops: &[PasteOp]) -> String {
    let mut out = String::new();
    for op in ops {
        if let PasteOp::Text(text) = op {
            out.push_str(text);
        }
    }
    out
}

pub fn has_keys(ops: &[PasteOp]) -> bool {
    ops.iter()
        .any(|op| matches!(op, PasteOp::Type(_) | PasteOp::Wait(_)))
}

pub fn has_type_token(text: &str) -> bool {
    walk_tokens(text, |inner| arg_after(inner, "type").is_some())
}

pub fn expand_template(text: &str, ctx: &Expand) -> String {
    expand_seen(text, ctx, &mut std::collections::HashSet::new())
}

fn expand_seen(
    text: &str,
    ctx: &Expand,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let text = apply_when(text, &ctx.app);
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if let Some(value) = token_value(inner.trim(), ctx, seen) {
                    out.push_str(&value);
                    i = close + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub(crate) fn find_close(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == '{' && chars[i + 1] == '{' {
            depth += 1;
            i += 2;
            continue;
        }
        if chars[i] == '}' && chars[i + 1] == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    None
}

/// `{{when chrome}}…{{when excel}}…{{when}}既定`。先頭の `{{when` より前は常に残す。
pub fn apply_when(text: &str, app: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    struct Mark {
        start: usize,
        end: usize,
        apps: Vec<String>,
    }
    let mut marks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                let token = inner.trim();
                if token == "when" || arg_after(token, "when").is_some() {
                    let arg = if token == "when" {
                        ""
                    } else {
                        arg_after(token, "when").unwrap_or("")
                    };
                    let apps: Vec<String> = arg
                        .split(|ch: char| ch == ',' || ch.is_whitespace())
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .map(str::to_string)
                        .collect();
                    marks.push(Mark {
                        start: i,
                        end: close + 2,
                        apps,
                    });
                    i = close + 2;
                    continue;
                }
                i = close + 2;
                continue;
            }
        }
        i += 1;
    }
    if marks.is_empty() {
        return text.to_string();
    }
    let prefix: String = chars[..marks[0].start].iter().collect();
    let mut chosen = None;
    let mut fallback = None;
    for (index, mark) in marks.iter().enumerate() {
        let body_end = marks
            .get(index + 1)
            .map(|next| next.start)
            .unwrap_or(chars.len());
        let body: String = chars[mark.end..body_end].iter().collect();
        if mark.apps.is_empty() {
            fallback = Some(body);
        } else if chosen.is_none()
            && mark
                .apps
                .iter()
                .any(|name| name.eq_ignore_ascii_case(app))
        {
            chosen = Some(body);
        }
    }
    let mut out = prefix;
    out.push_str(chosen.as_deref().or(fallback.as_deref()).unwrap_or(""));
    out
}

/// `{{sel|clip}}` の `|`。入れ子の `{{ }}` の中では切らない。
fn fallback_parts(inner: &str) -> Vec<String> {
    let chars: Vec<char> = inner.chars().collect();
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            depth += 1;
            buf.push(chars[i]);
            buf.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if depth > 0 && chars[i] == '}' && chars.get(i + 1) == Some(&'}') {
            depth -= 1;
            buf.push(chars[i]);
            buf.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if depth == 0 && chars[i] == '|' {
            parts.push(buf.trim().to_string());
            buf.clear();
            i += 1;
            continue;
        }
        buf.push(chars[i]);
        i += 1;
    }
    parts.push(buf.trim().to_string());
    parts
}

fn fallback_value(
    parts: &[String],
    ctx: &Expand,
    seen: &mut std::collections::HashSet<String>,
) -> String {
    let Some((head, rest)) = parts.split_first() else {
        return String::new();
    };
    if let Some(value) = token_value(head, ctx, seen) {
        if !value.is_empty() || rest.is_empty() {
            return value;
        }
        return fallback_value(rest, ctx, seen);
    }
    expand_seen(&parts.join("|"), ctx, seen)
}

fn token_value(
    inner: &str,
    ctx: &Expand,
    seen: &mut std::collections::HashSet<String>,
) -> Option<String> {
    let parts = fallback_parts(inner);
    if parts.len() > 1 {
        let head = token_value(&parts[0], ctx, seen)?;
        if !head.is_empty() {
            return Some(head);
        }
        return Some(fallback_value(&parts[1..], ctx, seen));
    }
    if inner == "date" {
        return Some(ctx.date.clone());
    }
    if inner == "time" {
        return Some(ctx.time.clone());
    }
    if inner == "clip" {
        return Some(ctx.clip.clone());
    }
    if inner == "sel" {
        return Some(ctx.sel.clone());
    }
    if let Some(name) = arg_after(inner, "ask") {
        return Some(
            ctx.answers
                .get(name)
                .cloned()
                .unwrap_or_default(),
        );
    }
    if inner == "uuid" {
        return Some(ctx.uuid.clone());
    }
    if inner == "user" {
        return Some(ctx.user.clone());
    }
    if inner == "host" {
        return Some(ctx.host.clone());
    }
    if inner == "app" {
        return Some(ctx.app.clone());
    }
    if inner == "front" {
        return Some(ctx.front.clone());
    }
    if let Some(spec) = arg_after(inner, "pick") {
        return Some(
            ctx.answers
                .get(&format!("pick:{spec}"))
                .cloned()
                .unwrap_or_default(),
        );
    }
    if let Some(name) = inner.strip_prefix('@') {
        let name = expand_seen(name.trim(), ctx, seen);
        let name = name.trim();
        if name.is_empty() {
            return Some(String::new());
        }
        if !seen.insert(name.to_string()) {
            return Some(String::new());
        }
        let body = ctx.aliases.get(name).cloned().unwrap_or_default();
        return Some(expand_seen(&body, ctx, seen));
    }
    if let Some(name) = arg_after(inner, "env") {
        let name = expand_seen(name, ctx, seen);
        let name = name.trim();
        if name.is_empty() {
            return Some(String::new());
        }
        return Some(std::env::var(name).unwrap_or_default());
    }
    if let Some(name) = arg_after(inner, "var") {
        let name = expand_seen(name, ctx, seen);
        let name = name.trim();
        if name.is_empty() {
            return Some(String::new());
        }
        let key = format!("var:{name}");
        if !seen.insert(key) {
            return Some(String::new());
        }
        let body = ctx.vars.get(name).cloned().unwrap_or_default();
        let expanded = expand_seen(&body, ctx, seen);
        if let Some(out) = crate::eval::colon_output(&expanded, &ctx.vars) {
            return Some(out);
        }
        return Some(expanded);
    }
    if let Some(name) = arg_after(inner, "tag") {
        let name = expand_seen(name, ctx, seen);
        let name = name.trim();
        if name.is_empty() {
            return Some(String::new());
        }
        let key = format!("tag:{name}");
        if !seen.insert(key) {
            return Some(String::new());
        }
        let body = ctx.tags.get(name).cloned().unwrap_or_default();
        return Some(expand_seen(&body, ctx, seen));
    }
    if inner == "n" {
        return Some(ctx.n.to_string());
    }
    if let Some(width) = arg_after(inner, "n") {
        let width: usize = width.parse().ok()?;
        return Some(format!("{:0width$}", ctx.n, width = width.min(8)));
    }
    if let Some(formatted) = date_token(inner, ctx) {
        return Some(formatted);
    }
    None
}

fn date_token(inner: &str, ctx: &Expand) -> Option<String> {
    if inner == "date" {
        return Some(ctx.date.clone());
    }
    let rest = inner.strip_prefix("date")?;
    if let Some((shift, fmt)) = parse_date_shift(rest) {
        let when = apply_date_shift(ctx.now, shift)?;
        return Some(match fmt {
            Some(fmt) if !fmt.is_empty() => when.format(fmt).to_string(),
            _ => when.format("%Y/%m/%d").to_string(),
        });
    }
    let fmt = arg_after(inner, "date")?;
    if fmt.is_empty() {
        return None;
    }
    Some(ctx.now.format(fmt).to_string())
}

/// `-1d` / `+2w` / `+1m:%Y%m` / `-1d %Y%m%d`
enum DateShift {
    Days(i64),
    Months(i64),
}

fn apply_date_shift(
    now: chrono::DateTime<chrono::Local>,
    shift: DateShift,
) -> Option<chrono::DateTime<chrono::Local>> {
    match shift {
        DateShift::Days(days) => Some(now + chrono::Duration::days(days)),
        DateShift::Months(months) if months >= 0 => {
            now.checked_add_months(chrono::Months::new(months as u32))
        }
        DateShift::Months(months) => now.checked_sub_months(chrono::Months::new((-months) as u32)),
    }
}

fn parse_date_shift(rest: &str) -> Option<(DateShift, Option<&str>)> {
    let rest = rest.trim();
    let (sign, rest) = if let Some(rest) = rest.strip_prefix('+') {
        (1i64, rest)
    } else if let Some(rest) = rest.strip_prefix('-') {
        (-1i64, rest)
    } else {
        return None;
    };
    let digits = rest
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits == 0 {
        return None;
    }
    let amount: i64 = rest[..digits].parse().ok()?;
    let after = rest[digits..].trim_start();
    let unit = after.chars().next()?;
    let after = after[unit.len_utf8()..].trim_start();
    let shift = match unit {
        'd' | 'D' => DateShift::Days(sign * amount),
        'w' | 'W' => DateShift::Days(sign * amount * 7),
        'm' | 'M' => DateShift::Months(sign * amount),
        _ => return None,
    };
    let fmt = if after.is_empty() {
        None
    } else if let Some(fmt) = after.strip_prefix(':') {
        Some(fmt.trim())
    } else {
        Some(after)
    };
    Some((shift, fmt.filter(|fmt| !fmt.is_empty())))
}

fn parse_wait(raw: &str) -> Option<u64> {
    let ms: u64 = raw.trim().parse().ok()?;
    Some(ms.min(5_000))
}

/// `{{var:a}}` と `{{var a}}` の両方。`:` はエクスプローラーの名前に使えない。
pub(crate) fn arg_after<'a>(inner: &'a str, name: &str) -> Option<&'a str> {
    let rest = inner.strip_prefix(name)?;
    let arg = if let Some(arg) = rest.strip_prefix(':') {
        arg
    } else if rest.starts_with(char::is_whitespace) {
        rest
    } else {
        return None;
    };
    Some(arg.trim())
}

/// タグ `app:chrome` / `app chrome` の値。空ならなし。
pub fn tag_arg<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let arg = arg_after(tag, name)?;
    if arg.is_empty() {
        None
    } else {
        Some(arg)
    }
}

/// ブラウザの `chrome|GitHub` からアプリ名だけ取る。
pub fn context_app(context: &str) -> &str {
    context.split('|').next().filter(|name| !name.is_empty()).unwrap_or(context)
}

/// Ctrl+Shift+n: `#slot:n` の先の行。無ければ位置（0始まり）。
pub fn ranked_index<T, F>(items: &[T], index: usize, tags: F) -> Option<usize>
where
    F: Fn(&T) -> &[String],
{
    let slot = (index + 1).to_string();
    items
        .iter()
        .position(|item| {
            tags(item)
                .iter()
                .any(|tag| tag_arg(tag, "slot") == Some(slot.as_str()))
        })
        .or_else(|| (index < items.len()).then_some(index))
}

fn token_has_sel(inner: &str) -> bool {
    fallback_parts(inner)
        .iter()
        .any(|part| part == "sel" || walk_tokens(part, token_has_sel))
}

pub fn has_sel_token(text: &str) -> bool {
    walk_tokens(text, token_has_sel)
}

pub fn has_sel_token_in(text: &str, app: &str) -> bool {
    walk_tokens(&apply_when(text, app), |inner| inner == "sel")
}

/// 出現順。同じ名前は 1 回だけ。
#[cfg_attr(not(test), allow(dead_code))]
pub fn ask_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    walk_tokens(text, |inner| {
        if let Some(name) = arg_after(inner, "ask") {
            if !name.is_empty() && !names.iter().any(|entry| entry == name) {
                names.push(name.to_string());
            }
        }
        false
    });
    names
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn pick_specs(text: &str) -> Vec<(String, Vec<String>)> {
    let mut specs = Vec::new();
    walk_tokens(text, |inner| {
        if let Some(spec) = arg_after(inner, "pick") {
            if !spec.is_empty() && !specs.iter().any(|(existing, _)| existing == spec) {
                if arg_after(spec, "tag").is_some_and(|name| !name.is_empty()) {
                    specs.push((spec.to_string(), Vec::new()));
                } else {
                    let options = spec
                        .split(',')
                        .map(|part| part.trim().to_string())
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>();
                    if !options.is_empty() {
                        specs.push((spec.to_string(), options));
                    }
                }
            }
        }
        false
    });
    specs
}

pub fn prefix_lines(text: &str, prefix: &str) -> String {
    let already = prefix_already(prefix);
    text.lines()
        .map(|line| {
            if line.starts_with(already) {
                line.to_string()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn prefix_already(prefix: &str) -> &str {
    let trimmed = prefix.trim_end();
    if trimmed.is_empty() {
        prefix
    } else {
        trimmed
    }
}

pub fn to_tsv(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.starts_with('[') {
        let value: serde_json::Value = serde_json::from_str(trimmed).ok()?;
        let arr = value.as_array()?;
        if arr.is_empty() {
            return None;
        }
        if arr.iter().all(|entry| entry.is_string()) {
            return Some(
                arr.iter()
                    .map(|entry| entry.as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join("\t"),
            );
        }
        if arr.iter().all(|entry| entry.is_object()) {
            let first = arr[0].as_object()?;
            let keys: Vec<String> = first.keys().cloned().collect();
            if keys.is_empty() {
                return None;
            }
            let same = arr.iter().all(|entry| {
                entry.as_object().map_or(false, |object| {
                    object.len() == keys.len() && keys.iter().all(|key| object.contains_key(key))
                })
            });
            if !same {
                return None;
            }
            let rows: Vec<String> = arr
                .iter()
                .map(|entry| {
                    let object = entry.as_object().unwrap();
                    keys.iter()
                        .map(|key| json_cell(&object[key]))
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect();
            return Some(rows.join("\n"));
        }
        return None;
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\t"))
    }
}

fn json_cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

pub fn log_path(tags: &[String]) -> Option<String> {
    tags.iter().find_map(|tag| {
        tag.strip_prefix("log:")
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToString::to_string)
    })
}

pub fn ttl_secs(tag: &str) -> Option<u64> {
    let rest = tag.strip_prefix("ttl:")?;
    if rest.len() < 2 {
        return None;
    }
    let (digits, unit) = rest.split_at(rest.len() - 1);
    let amount: u64 = digits.parse().ok()?;
    match unit {
        "m" => Some(amount.saturating_mul(60)),
        "h" => Some(amount.saturating_mul(3600)),
        "d" => Some(amount.saturating_mul(86400)),
        _ => None,
    }
}

pub fn expired(tags: &[String], created_at: u64, now: u64) -> bool {
    if created_at == 0 {
        return false;
    }
    tags.iter().any(|tag| {
        ttl_secs(tag).map_or(false, |secs| now.saturating_sub(created_at) >= secs)
    })
}

fn walk_tokens(text: &str, mut visit: impl FnMut(&str) -> bool) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if visit(inner.trim()) {
                    return true;
                }
                i = close + 2;
                continue;
            }
        }
        i += 1;
    }
    false
}

pub fn login_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default()
}

pub fn host_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_default()
}

/// `:format` 用の整形。行頭の引用符号を外し、余分な空行と前後の空白を落とす。
pub fn format_for_paste(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = strip_quote_marker(line);
        let line = line.trim_end().to_string();
        if line.is_empty() && lines.last().map_or(true, |last: &String| last.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    while lines.last().map_or(false, |last: &String| last.is_empty()) {
        lines.pop();
    }
    unwrap_quotes(lines.join("\n").trim())
}

fn strip_quote_marker(line: &str) -> &str {
    let mut rest = line.trim_start();
    let mut stripped = false;
    while let Some(next) = rest.strip_prefix('>') {
        rest = next.trim_start();
        stripped = true;
    }
    if stripped {
        rest
    } else {
        line
    }
}

fn unwrap_quotes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return text.to_string();
    }
    let pair = matches!(
        (chars[0], chars[chars.len() - 1]),
        ('"', '"') | ('\'', '\'') | ('`', '`') | ('“', '”') | ('「', '」') | ('『', '』')
    );
    if !pair {
        return text.to_string();
    }
    chars[1..chars.len() - 1]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// ブラウザのウィンドウタイトルから、ページを表す部分だけ取り出す。
#[cfg_attr(not(windows), allow(dead_code))]
pub fn browser_page(title: &str) -> Option<String> {
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    let title = strip_unread_count(title);
    for separator in [" - ", " — ", " – "] {
        if let Some(index) = title.rfind(separator) {
            let page = title[..index].trim();
            if !page.is_empty() {
                return Some(page.to_string());
            }
        }
    }
    Some(title.to_string())
}

fn strip_unread_count(title: &str) -> &str {
    let Some(rest) = title.strip_prefix('(') else {
        return title;
    };
    match rest.split_once(')') {
        Some((count, tail)) if !count.is_empty() && count.chars().all(|c| c.is_ascii_digit()) => {
            tail.trim_start()
        }
        _ => title,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_urls_and_multiline_and_paths() {
        assert_eq!(auto_tags("https://example.com"), vec!["url"]);
        assert!(auto_tags("a\nb").is_empty());
        assert_eq!(auto_tags(r"C:\work\memo.txt"), vec!["path"]);
        assert_eq!(auto_tags("/home/me/.bashrc"), vec!["path"]);
        assert!(auto_tags("ただの文章").is_empty());
    }

    #[test]
    fn tag_arg_reads_colon_or_space() {
        assert_eq!(tag_arg("app:chrome", "app"), Some("chrome"));
        assert_eq!(tag_arg("app chrome", "app"), Some("chrome"));
        assert_eq!(tag_arg("not:code", "not"), Some("code"));
        assert_eq!(tag_arg("app:", "app"), None);
        assert_eq!(tag_arg("app", "app"), None);
        assert_eq!(tag_arg("apple", "app"), None);
        assert_eq!(tag_arg("slot:3", "slot"), Some("3"));
        assert_eq!(tag_arg("slot 3", "slot"), Some("3"));
    }

    #[test]
    fn ranked_index_prefers_slot_then_position() {
        let items = vec![
            vec!["work".to_string()],
            vec!["slot:3".to_string()],
            vec!["slot:3".to_string()],
            vec!["a".to_string()],
        ];
        assert_eq!(ranked_index(&items, 2, |tags| tags.as_slice()), Some(1));
        assert_eq!(ranked_index(&items, 0, |tags| tags.as_slice()), Some(0));
        assert_eq!(ranked_index(&items, 1, |tags| tags.as_slice()), Some(1));
        assert_eq!(ranked_index(&items, 9, |tags| tags.as_slice()), None);
        let slot_first = vec![vec!["a".to_string()], vec!["slot:1".to_string()]];
        assert_eq!(ranked_index(&slot_first, 0, |tags| tags.as_slice()), Some(1));
    }

    #[test]
    fn context_app_strips_browser_page() {
        assert_eq!(context_app("chrome|GitHub"), "chrome");
        assert_eq!(context_app("code"), "code");
        assert_eq!(context_app("|page"), "|page");
    }

    #[test]
    fn a_url_is_not_also_a_path() {
        assert_eq!(auto_tags("https://example.com/a/b"), vec!["url"]);
    }

    fn sample_ctx() -> Expand {
        Expand {
            date: "2026/09/20".into(),
            time: "10:54".into(),
            clip: "CLIP".into(),
            sel: "SEL".into(),
            n: 3,
            uuid: "uuid-here".into(),
            user: "hatamon".into(),
            host: "pc".into(),
            app: "code".into(),
            front: "TODO.md".into(),
            now: chrono::Local::now(),
            answers: std::collections::HashMap::from([
                ("名前".into(), "hatamon".into()),
                ("pick:prod, stg".into(), "stg".into()),
            ]),
            aliases: std::collections::HashMap::from([
                ("foo".into(), "X{{date}}Y".into()),
                ("bar".into(), "BB{{@foo}}".into()),
                ("loop".into(), "{{@loop}}".into()),
            ]),
            vars: std::collections::HashMap::from([
                ("a".into(), "{{date}}".into()),
                ("b".into(), "a".into()),
                ("sum".into(), ":echo 2+3".into()),
                ("loop".into(), "{{var:loop}}".into()),
            ]),
            tags: std::collections::HashMap::from([
                ("work".into(), "alpha\nbeta".into()),
                ("dated".into(), "X{{date}}Y".into()),
                ("loop".into(), "{{tag:loop}}".into()),
                ("login".into(), "id{{type:<Tab>}}pass".into()),
            ]),
        }
    }

    #[test]
    fn expands_date_and_time() {
        let ctx = sample_ctx();
        assert_eq!(
            expand_template("{{date}} {{time}} 提出", &ctx),
            "2026/09/20 10:54 提出"
        );
        assert_eq!(expand_template("そのまま", &ctx), "そのまま");
    }

    #[test]
    fn expands_clip_n_user_host_and_padded_n() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{clip}}-{{n}}-{{n:2}}", &ctx), "CLIP-3-03");
        assert_eq!(expand_template("{{n 2}}", &ctx), "03");
        assert_eq!(expand_template("{{user}}@{{host}}", &ctx), "hatamon@pc");
    }

    #[test]
    fn expands_strftime_date() {
        let ctx = sample_ctx();
        assert_eq!(
            expand_template("{{date:%Y}}", &ctx),
            ctx.now.format("%Y").to_string()
        );
        assert_eq!(
            expand_template("{{date %Y}}", &ctx),
            ctx.now.format("%Y").to_string()
        );
        let yesterday = (ctx.now - chrono::Duration::days(1)).format("%Y/%m/%d").to_string();
        assert_eq!(expand_template("{{date-1d}}", &ctx), yesterday);
        assert_eq!(
            expand_template("{{date-1d:%Y%m%d}}", &ctx),
            (ctx.now - chrono::Duration::days(1)).format("%Y%m%d").to_string()
        );
        assert_eq!(
            expand_template("{{date -1d %Y%m%d}}", &ctx),
            (ctx.now - chrono::Duration::days(1)).format("%Y%m%d").to_string()
        );
        assert_eq!(
            expand_template("{{date+1w}}", &ctx),
            (ctx.now + chrono::Duration::days(7)).format("%Y/%m/%d").to_string()
        );
        assert_eq!(
            expand_template("{{date+1m:%Y-%m}}", &ctx),
            ctx.now
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .format("%Y-%m")
                .to_string()
        );
    }

    #[test]
    fn leaves_unknown_tokens_alone() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{nope}}", &ctx), "{{nope}}");
    }

    #[test]
    fn expands_sel_and_ask() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("**{{sel}}**", &ctx), "**SEL**");
        assert_eq!(expand_template("hi {{ask:名前}}", &ctx), "hi hatamon");
        assert_eq!(expand_template("hi {{ask 名前}}", &ctx), "hi hatamon");
        assert_eq!(ask_names("{{ask a}} {{ask:b}}"), vec!["a", "b"]);
        assert_eq!(expand_template("{{ask:missing}}", &ctx), "");
        assert!(has_sel_token("x {{sel}} y"));
        assert!(has_sel_token("{{sel|clip}}"));
        assert!(!has_sel_token("{{clip}}"));
        assert!(!has_sel_token("{{clip|front}}"));
        let mut empty_sel = sample_ctx();
        empty_sel.sel.clear();
        assert_eq!(expand_template("{{sel|clip}}", &empty_sel), "CLIP");
        assert_eq!(expand_template("{{sel|clip}}", &ctx), "SEL");
        let mut empty_front = sample_ctx();
        empty_front.front.clear();
        assert_eq!(expand_template("{{front|無題}}", &empty_front), "無題");
        assert_eq!(expand_template("{{front|無題}}", &ctx), "TODO.md");
        empty_sel.clip.clear();
        assert_eq!(expand_template("{{sel|clip|なし}}", &empty_sel), "なし");
        assert_eq!(expand_template("{{nope|clip}}", &ctx), "{{nope|clip}}");
        assert_eq!(
            expand_template("{{sel|hello {{date}}}}", &empty_sel),
            "hello 2026/09/20"
        );
        assert_eq!(ask_names("{{ask:a}} {{ask:b}} {{ask:a}}"), vec!["a", "b"]);
    }

    #[test]
    fn when_picks_matching_app_and_fallback() {
        let ctx = sample_ctx();
        assert_eq!(
            expand_template("id{{when chrome}}c{{when code}}k{{when}}x", &ctx),
            "idk"
        );
        assert_eq!(
            expand_template("{{when chrome}}c{{when excel}}e{{when}}d", &ctx),
            "d"
        );
        let mut excel = sample_ctx();
        excel.app = "EXCEL".into();
        assert_eq!(
            expand_template("{{when chrome, excel}}hit{{when}}miss", &excel),
            "hit"
        );
        assert_eq!(
            expand_template("keep {{date}}", &ctx),
            "keep 2026/09/20"
        );
        assert_eq!(apply_when("a{{whenever}}b", "code"), "a{{whenever}}b");
        assert!(!has_sel_token_in(
            "{{when chrome}}{{sel}}{{when}}plain",
            "code"
        ));
        assert!(has_sel_token_in(
            "{{when chrome}}{{sel}}{{when}}plain",
            "chrome"
        ));
    }

    #[test]
    fn expands_app_front_and_pick() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{app}}/{{front}}", &ctx), "code/TODO.md");
        assert_eq!(expand_template("{{pick:prod, stg}}", &ctx), "stg");
        assert_eq!(expand_template("{{pick prod, stg}}", &ctx), "stg");
        assert_eq!(
            pick_specs("{{pick a, b}} {{pick: a, b}}"),
            vec![("a, b".into(), vec!["a".into(), "b".into()])]
        );
        assert_eq!(
            pick_specs("{{pick: a, b}} {{pick: a, b}}"),
            vec![("a, b".into(), vec!["a".into(), "b".into()])]
        );
        assert_eq!(
            pick_specs("{{pick hata007@x, {{var:a}}}}"),
            vec![(
                "hata007@x, {{var:a}}".into(),
                vec!["hata007@x".into(), "{{var:a}}".into()]
            )]
        );
        assert_eq!(
            pick_specs("{{pick tag:env}} {{pick tag env}}"),
            vec![("tag:env".into(), vec![]), ("tag env".into(), vec![])]
        );
        let mut tagged = sample_ctx();
        tagged.answers.insert("pick:tag:env".into(), "stg".into());
        assert_eq!(expand_template("{{pick tag:env}}", &tagged), "stg");
        let mut nested = sample_ctx();
        nested.answers.insert(
            "pick:hata007@x, {{var:a}}".into(),
            "chose".into(),
        );
        assert_eq!(
            expand_template("{{pick hata007@x, {{var:a}}}}end", &nested),
            "choseend"
        );
    }

    #[test]
    fn expands_env_var() {
        let key = "HATACLIP_TEST_ENV_XYZ";
        unsafe { std::env::set_var(key, "env-value") };
        let ctx = sample_ctx();
        assert_eq!(
            expand_template("{{env:HATACLIP_TEST_ENV_XYZ}}", &ctx),
            "env-value"
        );
        assert_eq!(
            expand_template("{{env HATACLIP_TEST_ENV_XYZ}}", &ctx),
            "env-value"
        );
        assert_eq!(
            expand_template("{{env:  HATACLIP_TEST_ENV_XYZ  }}", &ctx),
            "env-value"
        );
        assert_eq!(expand_template("{{env:HATACLIP_NO_SUCH_VAR_ZZZ}}", &ctx), "");
        assert_eq!(expand_template("{{env:}}", &ctx), "");
        assert_eq!(expand_template("{{env}}", &ctx), "{{env}}");
    }

    #[test]
    fn expands_alias_nested_and_stops_cycles() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{@foo}}", &ctx), "X2026/09/20Y");
        assert_eq!(expand_template("{{@bar}}", &ctx), "BBX2026/09/20Y");
        assert_eq!(expand_template("{{@foo}} {{@foo}}", &ctx), "X2026/09/20Y ");
        assert_eq!(expand_template("{{@loop}}", &ctx), "");
        assert_eq!(expand_template("{{@missing}}", &ctx), "");
        assert_eq!(expand_template("{{@}}", &ctx), "");
        assert_eq!(expand_template("{{@ foo }}", &ctx), "X2026/09/20Y");
    }

    #[test]
    fn expands_var_nested_and_stops_cycles() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("date {{var:a}}", &ctx), "date 2026/09/20");
        assert_eq!(expand_template("date {{var a}}", &ctx), "date 2026/09/20");
        assert_eq!(expand_template("{{var}}", &ctx), "{{var}}");
        assert_eq!(expand_template("{{var:sum}}", &ctx), "5");
        assert_eq!(expand_template("{{var:loop}}", &ctx), "");
        assert_eq!(expand_template("{{var:missing}}", &ctx), "");
        assert_eq!(expand_template("{{var:}}", &ctx), "");
        assert_eq!(expand_template("{{var: {{var: b}}}}", &ctx), "2026/09/20");
        assert_eq!(expand_template("{{var:{{var:b}}}}", &ctx), "2026/09/20");
        assert_eq!(expand_template("{{var {{var b}}}}", &ctx), "2026/09/20");
    }

    #[test]
    fn expands_tag_nested_and_stops_cycles() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{tag:work}}", &ctx), "alpha\nbeta");
        assert_eq!(expand_template("{{tag work}}", &ctx), "alpha\nbeta");
        assert_eq!(expand_template("{{tag:dated}}", &ctx), "X2026/09/20Y");
        assert_eq!(expand_template("{{tag:loop}}", &ctx), "");
        assert_eq!(expand_template("{{tag:missing}}", &ctx), "");
        assert_eq!(expand_template("{{tag:}}", &ctx), "");
        assert_eq!(expand_template("{{tag}}", &ctx), "{{tag}}");
        assert_eq!(
            expand_template("{{tag:login}}", &ctx),
            "id{{type:<Tab>}}pass"
        );
    }

    #[test]
    fn splits_type_tokens_into_ops() {
        use crate::keys::{TypeAtom, TypeKey, TypeStep};
        let tab = TypeAtom::Key(TypeStep {
            ctrl: false,
            shift: false,
            alt: false,
            key: TypeKey::Tab,
        });
        let ctx = sample_ctx();
        assert_eq!(
            expand_ops("id{{type:<Tab>}}pass", &ctx),
            vec![
                PasteOp::Text("id".into()),
                PasteOp::Type(vec![tab.clone()]),
                PasteOp::Text("pass".into()),
            ]
        );
        assert_eq!(
            expand_ops("{{tag:login}}", &ctx),
            vec![
                PasteOp::Text("id".into()),
                PasteOp::Type(vec![tab]),
                PasteOp::Text("pass".into()),
            ]
        );
        assert_eq!(
            expand_ops("{{type:<Ctrl+A>abc<Enter>}}", &ctx),
            vec![PasteOp::Type(crate::keys::parse_type_script(
                "<Ctrl+A>abc<Enter>"
            ))]
        );
        assert_eq!(expand_ops("plain", &ctx), vec![PasteOp::Text("plain".into())]);
        assert_eq!(
            expand_ops("a{{wait:200}}b", &ctx),
            vec![
                PasteOp::Text("a".into()),
                PasteOp::Wait(200),
                PasteOp::Text("b".into()),
            ]
        );
        assert!(has_type_token("id{{type:<Tab>}}pass"));
        assert!(has_type_token("{{type <Tab>}}"));
        assert!(!has_type_token("{{date}}"));
    }

    #[test]
    fn prefixes_lines_unless_already_marked() {
        assert_eq!(prefix_lines("a\nb", "> "), "> a\n> b");
        assert_eq!(prefix_lines("> a\nb", "> "), "> a\n> b");
        assert_eq!(prefix_lines("a\n* b", "* "), "* a\n* b");
        assert_eq!(prefix_lines("- [ ] a\nb", "- [ ] "), "- [ ] a\n- [ ] b");
    }

    #[test]
    fn tsv_from_json_and_lines() {
        assert_eq!(to_tsv("[\"a\",\"b\"]").as_deref(), Some("a\tb"));
        assert_eq!(
            to_tsv("[{\"x\":1,\"y\":\"z\"}]").as_deref(),
            Some("1\tz")
        );
        assert_eq!(to_tsv("one\ntwo").as_deref(), Some("one\ttwo"));
        assert_eq!(to_tsv("[]"), None);
        assert_eq!(to_tsv("[1,2]"), None);
    }

    #[test]
    fn ttl_parses_units_and_expiry() {
        assert_eq!(ttl_secs("ttl:30m"), Some(1800));
        assert_eq!(ttl_secs("ttl:1h"), Some(3600));
        assert_eq!(ttl_secs("ttl:1d"), Some(86400));
        assert_eq!(ttl_secs("ttl:nope"), None);
        assert!(expired(&["ttl:1h".into()], 10, 10 + 3600));
        assert!(!expired(&["ttl:1h".into()], 10, 10 + 3599));
        assert!(!expired(&["ttl:1h".into()], 0, 10_000));
    }

    #[test]
    fn formatting_drops_quote_markers_and_blank_runs() {
        assert_eq!(
            format_for_paste("> hello\n> > world\n\n\n> bye\n"),
            "hello\nworld\n\nbye"
        );
    }

    #[test]
    fn formatting_keeps_indentation_inside() {
        assert_eq!(
            format_for_paste("\n  fn main() {\n      body\n  }\n\n"),
            "fn main() {\n      body\n  }"
        );
    }

    #[test]
    fn formatting_unwraps_matching_quotes_only() {
        assert_eq!(format_for_paste("\"quoted\""), "quoted");
        assert_eq!(format_for_paste("「かぎ括弧」"), "かぎ括弧");
        assert_eq!(format_for_paste("\"half"), "\"half");
    }

    #[test]
    fn reads_the_page_out_of_a_browser_title() {
        assert_eq!(
            browser_page("hataclip - GitHub - Google Chrome").as_deref(),
            Some("hataclip - GitHub")
        );
        assert_eq!(
            browser_page("(3) 受信トレイ — Mozilla Firefox").as_deref(),
            Some("受信トレイ")
        );
        assert_eq!(browser_page("   ").as_deref(), None);
    }
}
