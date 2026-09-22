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
    pub focus: String,
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
    let text = apply_when(text, &when_env(ctx));
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

/// 貼るときの `{{when}}` 判定。`app` と `focus` が空ならその枝は当たらない。
pub struct WhenEnv<'a> {
    pub app: &'a str,
    pub focus: &'a str,
    pub vars: &'a std::collections::HashMap<String, String>,
}

fn when_env(ctx: &Expand) -> WhenEnv<'_> {
    WhenEnv {
        app: &ctx.app,
        focus: &ctx.focus,
        vars: &ctx.vars,
    }
}

enum WhenKind {
    App(Vec<String>),
    Var { name: String, value: String },
    Focus(String),
    Fallback,
}

/// `{{when app:}}` `{{when var:}}` `{{when focus:}}` `{{when}}`。先に当たった枝だけ残す。
/// `{{when chrome}}` は枝にしない。先頭の `{{when` より前は常に残す。
pub fn apply_when(text: &str, env: &WhenEnv<'_>) -> String {
    let chars: Vec<char> = text.chars().collect();
    struct Mark {
        start: usize,
        end: usize,
        kind: WhenKind,
    }
    let mut marks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if let Some(kind) = when_kind(inner.trim()) {
                    marks.push(Mark {
                        start: i,
                        end: close + 2,
                        kind,
                    });
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
        match &mark.kind {
            WhenKind::Fallback => fallback = Some(body),
            _ if chosen.is_none() && when_hits(&mark.kind, env) => chosen = Some(body),
            _ => {}
        }
    }
    let mut out = prefix;
    out.push_str(chosen.as_deref().or(fallback.as_deref()).unwrap_or(""));
    out
}

fn when_kind(token: &str) -> Option<WhenKind> {
    if token == "when" {
        return Some(WhenKind::Fallback);
    }
    let arg = arg_after(token, "when")?;
    let arg = arg.trim();
    if let Some(rest) = arg.strip_prefix("app:") {
        let names: Vec<String> = rest
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();
        if names.is_empty() {
            return None;
        }
        return Some(WhenKind::App(names));
    }
    if let Some(rest) = arg.strip_prefix("var:") {
        return when_var(rest.trim());
    }
    if let Some(rest) = arg.strip_prefix("focus:") {
        let name = rest.trim();
        if name.is_empty() {
            return None;
        }
        return Some(WhenKind::Focus(name.to_string()));
    }
    None
}

fn when_var(rest: &str) -> Option<WhenKind> {
    let (name, raw) = rest.split_once(':')?;
    let name = name.trim();
    if !is_ident(name) {
        return None;
    }
    let value = unquote_double(raw.trim())?;
    Some(WhenKind::Var {
        name: name.to_string(),
        value,
    })
}

fn when_hits(kind: &WhenKind, env: &WhenEnv<'_>) -> bool {
    match kind {
        WhenKind::App(names) => names
            .iter()
            .any(|name| !env.app.is_empty() && name.eq_ignore_ascii_case(env.app)),
        WhenKind::Var { name, value } => env.vars.get(name).is_some_and(|current| current == value),
        WhenKind::Focus(name) => !env.focus.is_empty() && name.eq_ignore_ascii_case(env.focus),
        WhenKind::Fallback => false,
    }
}

pub(crate) fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// `"` で括った値。`\"` は `"`、`\t` はタブ、`\n` は改行。閉じたあとに文字があればなし。
pub(crate) fn unquote_double(raw: &str) -> Option<String> {
    let text = raw.trim();
    let mut chars = text.chars();
    if chars.next() != Some('"') {
        return None;
    }
    let body: Vec<char> = chars.collect();
    let mut out = String::new();
    let mut escaped = false;
    for (index, ch) in body.iter().copied().enumerate() {
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
        if ch == '"' {
            let tail: String = body[index + 1..].iter().collect();
            if !tail.trim().is_empty() {
                return None;
            }
            return Some(out);
        }
        out.push(ch);
    }
    None
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
    if inner == "nop" || arg_after(inner, "nop").is_some() {
        return Some(String::new());
    }
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
    if inner == "focus" {
        return Some(ctx.focus.clone());
    }
    if let Some(spec) = arg_after(inner, "pick") {
        if pick_kind(spec).is_none() {
            return Some(String::new());
        }
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

/// 本文と差し込んだエイリアスから、書いてある `{{var:名前}}`。
pub fn referenced_vars(
    text: &str,
    env: &WhenEnv<'_>,
    aliases: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::HashSet::new();
    collect_vars(&apply_when(text, env), env, aliases, &mut seen, &mut names);
    names
}

fn collect_vars(
    text: &str,
    env: &WhenEnv<'_>,
    aliases: &std::collections::HashMap<String, String>,
    seen_alias: &mut std::collections::HashSet<String>,
    names: &mut Vec<String>,
) {
    walk_tokens(text, |inner| {
        for part in fallback_parts(inner) {
            if let Some(name) = arg_after(&part, "var") {
                let name = name.trim();
                if !name.is_empty()
                    && !name.contains('{')
                    && !names.iter().any(|existing| existing == name)
            {
                    names.push(name.to_string());
                }
            }
            if let Some(alias) = part.strip_prefix('@') {
                let alias = alias.trim();
                if !alias.is_empty() && !alias.contains('{') && seen_alias.insert(alias.to_string())
                {
                    if let Some(body) = aliases.get(alias) {
                        collect_vars(&apply_when(body, env), env, aliases, seen_alias, names);
                    }
                }
            }
            if part.contains("{{") && walk_tokens(&part, |_| true) {
                collect_vars(&part, env, aliases, seen_alias, names);
            }
        }
        false
    });
}

pub fn has_sel_token_in(text: &str, env: &WhenEnv<'_>) -> bool {
    has_sel_token(&apply_when(text, env))
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

fn pick_kind(spec: &str) -> Option<()> {
    if spec.starts_with("list:") {
        let options = spec[5..]
            .split(',')
            .map(str::trim)
            .any(|part| !part.is_empty());
        return options.then_some(());
    }
    if arg_after(spec, "tag").is_some_and(|name| !name.is_empty()) {
        return Some(());
    }
    if let Some(raw) = spec.strip_prefix("search:") {
        return unquote_double(raw).filter(|query| !query.is_empty()).map(|_| ());
    }
    None
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn pick_specs(text: &str) -> Vec<(String, Vec<String>)> {
    let mut specs = Vec::new();
    walk_tokens(text, |inner| {
        if let Some(spec) = arg_after(inner, "pick") {
            if pick_kind(spec).is_some() && !specs.iter().any(|(existing, _)| existing == spec) {
                let options = if let Some(rest) = spec.strip_prefix("list:") {
                    rest.split(',')
                        .map(|part| part.trim().to_string())
                        .filter(|part| !part.is_empty())
                        .collect()
                } else {
                    Vec::new()
                };
                specs.push((spec.to_string(), options));
            }
        }
        false
    });
    specs
}

pub fn prefix_lines(text: &str, prefix: &str) -> String {
    affix_lines(text, prefix, "")
}

/// 行頭と行末。印が空白だけならその空白で比べ、そうでなければ端の空白を除いて比べる。
pub fn affix_lines(text: &str, prefix: &str, suffix: &str) -> String {
    let start = if prefix.trim_end().is_empty() {
        prefix
    } else {
        prefix.trim_end()
    };
    let end = if suffix.trim_start().is_empty() {
        suffix
    } else {
        suffix.trim_start()
    };
    text.lines()
        .map(|line| affix_line(line, prefix, suffix, start, end))
        .collect::<Vec<_>>()
        .join("\n")
}

fn affix_line(line: &str, prefix: &str, suffix: &str, start: &str, end: &str) -> String {
    let mut out = String::new();
    if !prefix.is_empty() && !line.starts_with(start) {
        out.push_str(prefix);
    }
    out.push_str(line);
    if !suffix.is_empty() && !line.ends_with(end) {
        out.push_str(suffix);
    }
    out
}

pub fn split_quote_arg(arg: &str) -> (&str, &str) {
    arg.split_once('\u{1}').unwrap_or((arg, ""))
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

/// 名前は `_` `-` 空白と大文字の境目で切る。`upper` / `lower` は文字だけ。
pub fn recase(text: &str, style: &str) -> String {
    match style {
        "upper" => text.to_uppercase(),
        "lower" => text.to_lowercase(),
        "camel" | "pascal" | "snake" | "kebab" => text
            .lines()
            .map(|line| recase_line(line, style))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => text.to_string(),
    }
}

fn recase_line(line: &str, style: &str) -> String {
    let words = ident_words(line);
    if words.is_empty() {
        return String::new();
    }
    match style {
        "snake" => words.join("_"),
        "kebab" => words.join("-"),
        "pascal" => words.into_iter().map(|word| capitalize(&word)).collect(),
        "camel" => {
            let mut out = String::new();
            for (index, word) in words.into_iter().enumerate() {
                if index == 0 {
                    out.push_str(&word);
                } else {
                    out.push_str(&capitalize(&word));
                }
            }
            out
        }
        _ => line.to_string(),
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_uppercase().to_string();
    out.extend(chars);
    out
}

fn ident_words(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut words = Vec::new();
    let mut buf = String::new();
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch == '_' || ch == '-' || ch.is_whitespace() {
            push_word(&mut words, &mut buf);
            continue;
        }
        let next = chars.get(index + 1).copied();
        if !buf.is_empty() && ch.is_uppercase() {
            let prev = buf.chars().last().unwrap();
            if !prev.is_uppercase() || next.is_some_and(|item| item.is_lowercase()) {
                push_word(&mut words, &mut buf);
            }
        }
        buf.push(ch);
    }
    push_word(&mut words, &mut buf);
    words
        .into_iter()
        .map(|word| word.to_lowercase())
        .collect()
}

fn push_word(words: &mut Vec<String>, buf: &mut String) {
    if !buf.is_empty() {
        words.push(std::mem::take(buf));
    }
}

/// 各行を区切りで分け、列をタブ1つでつなぎ直す。
pub fn split_fields(text: &str, sep: &str) -> String {
    if sep.is_empty() {
        return text.to_string();
    }
    text.lines()
        .map(|line| line.split(sep).collect::<Vec<_>>().join("\t"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 1 始まりの列。足りない行は捨てる。1 未満、または残る行が無いときはなし。
pub fn take_column(text: &str, index: i64) -> Option<String> {
    if index < 1 {
        return None;
    }
    let nth = usize::try_from(index - 1).ok()?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if let Some(col) = line.split('\t').nth(nth) {
            rows.push(col.to_string());
        }
    }
    if rows.is_empty() {
        None
    } else {
        Some(rows.join("\n"))
    }
}

/// 左にあって右に無い行は `- `、右にあって左に無い行は `+ `。全部同じならなし。
pub fn line_diff(left: &str, right: &str) -> Option<String> {
    let right_set: std::collections::HashSet<&str> = right.lines().collect();
    let left_set: std::collections::HashSet<&str> = left.lines().collect();
    let mut rows = Vec::new();
    for line in left.lines() {
        if !right_set.contains(line) {
            rows.push(format!("- {line}"));
        }
    }
    for line in right.lines() {
        if !left_set.contains(line) {
            rows.push(format!("+ {line}"));
        }
    }
    if rows.is_empty() {
        None
    } else {
        Some(rows.join("\n"))
    }
}

/// 左にあって右に無い行。0行ならなし。
pub fn only_lines(left: &str, right: &str) -> Option<String> {
    let right_set: std::collections::HashSet<&str> = right.lines().collect();
    let rows: Vec<&str> = left.lines().filter(|line| !right_set.contains(line)).collect();
    if rows.is_empty() {
        None
    } else {
        Some(rows.join("\n"))
    }
}

/// JSON Pointer の値。文字列は引用符なし。オブジェクトと配列は空白なしの JSON。
pub fn json_at(text: &str, pointer: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text.trim()).ok()?;
    Some(json_scalar(value.pointer(pointer)?))
}

/// ポインタの位置を置き換える。無い、または JSON でなければなし。
pub fn json_put(text: &str, pointer: &str, raw: &str) -> Option<String> {
    let mut value: serde_json::Value = serde_json::from_str(text.trim()).ok()?;
    let incoming = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) {
        parsed
    } else {
        serde_json::Value::String(raw.to_string())
    };
    let slot = value.pointer_mut(pointer)?;
    *slot = incoming;
    serde_json::to_string(&value).ok()
}

pub fn json_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Null => "null".into(),
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::Number(number) => number.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// 読める JSON を字下げして返す。読めなければなし。
pub fn pretty_json(text: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(text.trim()).ok()?;
    serde_json::to_string_pretty(&value).ok()
}

/// 読める XML を字下げして返す。タグが閉じていなければなし。
pub fn pretty_xml(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with('<') {
        return None;
    }
    let tokens = xml_tokens(trimmed)?;
    let mut stack: Vec<String> = Vec::new();
    for token in &tokens {
        match token {
            XmlToken::Open(raw) => stack.push(xml_name(raw)?.to_string()),
            XmlToken::Close(raw) => {
                let name = xml_name(raw)?;
                if stack.pop().as_deref() != Some(name) {
                    return None;
                }
            }
            XmlToken::Empty(raw) => {
                xml_name(raw)?;
            }
            XmlToken::Other(_) | XmlToken::Text(_) => {}
        }
    }
    if !stack.is_empty() {
        return None;
    }
    Some(render_xml(&tokens))
}

enum XmlToken {
    Open(String),
    Close(String),
    Empty(String),
    Other(String),
    Text(String),
}

fn xml_tokens(text: &str) -> Option<Vec<XmlToken>> {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    let mut tokens = Vec::new();
    while index < chars.len() {
        if chars[index] != '<' {
            let start = index;
            while index < chars.len() && chars[index] != '<' {
                index += 1;
            }
            let raw: String = chars[start..index].iter().collect();
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                tokens.push(XmlToken::Text(trimmed.to_string()));
            }
            continue;
        }
        let (raw, next) = read_markup(&chars, index)?;
        index = next;
        tokens.push(classify_markup(&raw));
    }
    Some(tokens)
}

fn read_markup(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars.get(start) != Some(&'<') {
        return None;
    }
    if chars.get(start + 1) == Some(&'!')
        && chars.get(start + 2) == Some(&'-')
        && chars.get(start + 3) == Some(&'-')
    {
        let mut index = start + 4;
        while index + 2 < chars.len() {
            if chars[index] == '-' && chars[index + 1] == '-' && chars[index + 2] == '>' {
                let raw: String = chars[start..index + 3].iter().collect();
                return Some((raw, index + 3));
            }
            index += 1;
        }
        return None;
    }
    if chars.get(start + 1) == Some(&'!')
        && chars.get(start + 2) == Some(&'[')
        && chars[start..].starts_with(&['<', '!', '[', 'C', 'D', 'A', 'T', 'A', '['])
    {
        let mut index = start + 9;
        while index + 2 < chars.len() {
            if chars[index] == ']' && chars[index + 1] == ']' && chars[index + 2] == '>' {
                let raw: String = chars[start..index + 3].iter().collect();
                return Some((raw, index + 3));
            }
            index += 1;
        }
        return None;
    }
    let mut index = start + 1;
    let mut quote: Option<char> = None;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(mark) = quote {
            if ch == mark {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            let raw: String = chars[start..index + 1].iter().collect();
            return Some((raw, index + 1));
        }
        index += 1;
    }
    None
}

fn classify_markup(raw: &str) -> XmlToken {
    let trimmed = raw.trim();
    if trimmed.starts_with("<?") || trimmed.starts_with("<!") {
        return XmlToken::Other(trimmed.to_string());
    }
    if trimmed.starts_with("</") {
        return XmlToken::Close(trimmed.to_string());
    }
    if trimmed.ends_with("/>") {
        return XmlToken::Empty(trimmed.to_string());
    }
    XmlToken::Open(trimmed.to_string())
}

fn xml_name(raw: &str) -> Option<&str> {
    let inner = raw
        .trim()
        .trim_start_matches('<')
        .trim_start_matches('/')
        .trim_end_matches('>')
        .trim_end_matches('/')
        .trim();
    let name = inner.split_whitespace().next().filter(|name| !name.is_empty())?;
    Some(name)
}

fn render_xml(tokens: &[XmlToken]) -> String {
    let mut depth = 0i32;
    let mut lines = Vec::new();
    for token in tokens {
        let (text, before, after) = match token {
            XmlToken::Close(raw) => (raw.as_str(), true, false),
            XmlToken::Open(raw) => (raw.as_str(), false, true),
            XmlToken::Empty(raw) | XmlToken::Other(raw) | XmlToken::Text(raw) => {
                (raw.as_str(), false, false)
            }
        };
        if before {
            depth -= 1;
        }
        let pad = "  ".repeat(depth.max(0) as usize);
        lines.push(format!("{pad}{text}"));
        if after {
            depth += 1;
        }
    }
    lines.join("\n")
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
            focus: "Edit".into(),
            now: chrono::Local::now(),
            answers: std::collections::HashMap::from([
                ("名前".into(), "hatamon".into()),
                ("pick:list: prod, stg".into(), "stg".into()),
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
        assert_eq!(expand_template("hello {{nop: {{date}}}} world", &ctx), "hello  world");
        assert_eq!(expand_template("{{nop: a|b}}", &ctx), "");
        assert_eq!(expand_template("{{nope}}", &ctx), "{{nope}}");
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
        let empty_vars = std::collections::HashMap::new();
        assert!(has_sel_token_in("{{sel|clip}}", &bare("code", &empty_vars)));
        assert!(has_sel_token_in(
            "{{when app: chrome}}{{sel|clip}}{{when}}",
            &bare("chrome", &empty_vars)
        ));
        assert!(!has_sel_token_in(
            "{{when app: chrome}}{{sel|clip}}{{when}}",
            &bare("code", &empty_vars)
        ));
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
        let aliases = std::collections::HashMap::from([("foo".into(), "{{var:a}}".into())]);
        let empty_vars = std::collections::HashMap::new();
        assert_eq!(
            referenced_vars("{{var:b}} {{@foo}}", &bare("code", &empty_vars), &aliases),
            vec!["b".to_string(), "a".to_string()]
        );
        assert!(referenced_vars(
            "{{when app: excel}}{{var:a}}{{when}}",
            &bare("code", &empty_vars),
            &aliases
        )
        .is_empty());
        assert_eq!(
            referenced_vars("{{var:a|なし}}", &bare("code", &empty_vars), &std::collections::HashMap::new()),
            vec!["a".to_string()]
        );
        assert_eq!(ask_names("{{ask:a}} {{ask:b}} {{ask:a}}"), vec!["a", "b"]);
    }

    #[test]
    fn when_picks_matching_app_and_fallback() {
        let ctx = sample_ctx();
        let empty_vars = std::collections::HashMap::new();
        assert_eq!(
            expand_template("id{{when app: chrome}}c{{when app: code}}k{{when}}x", &ctx),
            "idk"
        );
        assert_eq!(
            expand_template("{{when app: chrome}}c{{when app: excel}}e{{when}}d", &ctx),
            "d"
        );
        assert_eq!(
            expand_template("{{when chrome}}c{{when}}d", &ctx),
            "{{when chrome}}cd"
        );
        let mut excel = sample_ctx();
        excel.app = "EXCEL".into();
        assert_eq!(
            expand_template("{{when app: chrome, msedge}}hit{{when}}miss", &excel),
            "miss"
        );
        excel.app = "msedge".into();
        assert_eq!(
            expand_template("{{when app: chrome, msedge}}hit{{when}}miss", &excel),
            "hit"
        );
        let mut named = sample_ctx();
        named.vars.insert("a".into(), "AAA".into());
        assert_eq!(
            expand_template(
                r#"{{when var: a: "AAA"}}git{{when var: a: "BBB"}}hg{{when}}none"#,
                &named
            ),
            "git"
        );
        assert_eq!(
            expand_template(
                r#"{{when var: a: "say \"hi\""}}yes{{when}}no"#,
                &named
            ),
            "no"
        );
        named.vars.insert("a".into(), "say \"hi\"".into());
        assert_eq!(
            expand_template(
                r#"{{when var: a: "say \"hi\""}}yes{{when}}no"#,
                &named
            ),
            "yes"
        );
        assert_eq!(
            expand_template("{{when focus: Edit}}box{{when}}other", &ctx),
            "box"
        );
        let mut doc = sample_ctx();
        doc.focus = "Document".into();
        assert_eq!(
            expand_template("{{when app: chrome}}web{{when focus: Edit}}in{{when}}out", &doc),
            "out"
        );
        assert_eq!(expand_template("{{focus}}", &ctx), "Edit");
        let mut blank = sample_ctx();
        blank.app.clear();
        blank.focus.clear();
        assert_eq!(
            expand_template("{{when app: chrome}}web{{when focus: Edit}}in{{when}}none", &blank),
            "none"
        );
        assert_eq!(
            expand_template("keep {{date}}", &ctx),
            "keep 2026/09/20"
        );
        assert_eq!(
            apply_when("a{{whenever}}b", &bare("code", &empty_vars)),
            "a{{whenever}}b"
        );
        assert!(!has_sel_token_in(
            "{{when app: chrome}}{{sel}}{{when}}plain",
            &bare("code", &empty_vars)
        ));
        assert!(has_sel_token_in(
            "{{when app: chrome}}{{sel}}{{when}}plain",
            &bare("chrome", &empty_vars)
        ));
    }

    fn bare<'a>(
        app: &'a str,
        vars: &'a std::collections::HashMap<String, String>,
    ) -> WhenEnv<'a> {
        WhenEnv {
            app,
            focus: "",
            vars,
        }
    }

    #[test]
    fn expands_app_front_and_pick() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{app}}/{{front}}", &ctx), "code/TODO.md");
        assert_eq!(expand_template("{{pick:prod, stg}}", &ctx), "");
        assert_eq!(expand_template("{{pick list: prod, stg}}", &ctx), "stg");
        assert_eq!(pick_specs("{{pick: a, b}}"), Vec::<(String, Vec<String>)>::new());
        assert_eq!(
            pick_specs("{{pick list: a, b}} {{pick list: a, b}}"),
            vec![("list: a, b".into(), vec!["a".into(), "b".into()])]
        );
        assert_eq!(
            pick_specs("{{pick list: hata007@x, {{var:a}}}}"),
            vec![(
                "list: hata007@x, {{var:a}}".into(),
                vec!["hata007@x".into(), "{{var:a}}".into()]
            )]
        );
        assert_eq!(
            pick_specs("{{pick tag:env}} {{pick tag env}}"),
            vec![("tag:env".into(), vec![]), ("tag env".into(), vec![])]
        );
        assert_eq!(
            pick_specs(r#"{{pick search: "xx"}} {{pick search: xx}}"#),
            vec![(r#"search: "xx""#.into(), vec![])]
        );
        let mut tagged = sample_ctx();
        tagged.answers.insert("pick:tag:env".into(), "stg".into());
        assert_eq!(expand_template("{{pick tag:env}}", &tagged), "stg");
        let mut nested = sample_ctx();
        nested.answers.insert(
            "pick:list: hata007@x, {{var:a}}".into(),
            "chose".into(),
        );
        assert_eq!(
            expand_template("{{pick list: hata007@x, {{var:a}}}}end", &nested),
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
        assert_eq!(affix_lines("hello", "> ", "<"), "> hello<");
        assert_eq!(affix_lines("> hello", "> ", "<"), "> hello<");
        assert_eq!(affix_lines("hello<", "> ", "<"), "> hello<");
        assert_eq!(affix_lines("> hello<", "> ", "<"), "> hello<");
        assert_eq!(affix_lines("a\n\nb", "> ", "<"), "> a<\n> <\n> b<");
        assert_eq!(affix_lines("hello", "", "<"), "hello<");
        assert_eq!(affix_lines("hello  ", "> ", "  "), "> hello  ");
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
    fn recase_splits_on_separators_and_capitals() {
        assert_eq!(recase("foo_bar", "camel"), "fooBar");
        assert_eq!(recase("foo bar", "pascal"), "FooBar");
        assert_eq!(recase("fooBar", "snake"), "foo_bar");
        assert_eq!(recase("fooBar", "kebab"), "foo-bar");
        assert_eq!(recase("HTTPResponse", "snake"), "http_response");
        assert_eq!(recase("XMLParser", "kebab"), "xml-parser");
        assert_eq!(recase("Foo_Bar\nbaz", "camel"), "fooBar\nbaz");
        assert_eq!(recase("AbC", "upper"), "ABC");
        assert_eq!(recase("AbC", "lower"), "abc");
    }

    #[test]
    fn split_col_and_json_pointer() {
        assert_eq!(split_fields("a,b,c", ","), "a\tb\tc");
        assert_eq!(split_fields("a\tb\tc", "\t"), "a\tb\tc");
        assert_eq!(take_column("a\tb\tc\nd", 2).as_deref(), Some("b"));
        assert_eq!(take_column("a", 2), None);
        assert_eq!(take_column("a\tb", 0), None);
        assert_eq!(
            json_at(r#"{"items":[{"name":"hatamon"}]}"#, "/items/0/name").as_deref(),
            Some("hatamon")
        );
        assert_eq!(json_at(r#"{"n":1,"ok":true,"x":null}"#, "/n").as_deref(), Some("1"));
        assert_eq!(json_at(r#"{"ok":true}"#, "/ok").as_deref(), Some("true"));
        assert_eq!(json_at(r#"{"x":null}"#, "/x").as_deref(), Some("null"));
        assert_eq!(json_at(r#"{"a":{"b":1}}"#, "/a").as_deref(), Some(r#"{"b":1}"#));
        assert_eq!(json_at("nope", "/a"), None);
        assert_eq!(json_at(r#"{"a":1}"#, "/missing"), None);
        assert_eq!(line_diff("a\nb", "b\nc").as_deref(), Some("- a\n+ c"));
        assert_eq!(line_diff("a\nb", "a\nb"), None);
        assert_eq!(only_lines("b\nc", "a\nb").as_deref(), Some("c"));
        assert_eq!(only_lines("a", "a"), None);
        let put = json_put(r#"{"name":"x","n":1}"#, "/n", "2").unwrap();
        let put: serde_json::Value = serde_json::from_str(&put).unwrap();
        assert_eq!(put["n"], 2);
        assert_eq!(put["name"], "x");
        let named = json_put(r#"{"name":"x"}"#, "/name", "hello").unwrap();
        let named: serde_json::Value = serde_json::from_str(&named).unwrap();
        assert_eq!(named["name"], "hello");
        assert_eq!(json_put(r#"{"n":1}"#, "/missing", "2"), None);
        assert_eq!(json_put("nope", "/n", "2"), None);
    }

    #[test]
    fn pretty_json_and_xml_indent_or_refuse() {
        assert_eq!(pretty_json(r#"{"a":1}"#).as_deref(), Some("{\n  \"a\": 1\n}"));
        assert!(pretty_json("{").is_none());
        assert_eq!(
            pretty_xml("<root><child>hi</child><empty/></root>").as_deref(),
            Some("<root>\n  <child>\n    hi\n  </child>\n  <empty/>\n</root>")
        );
        assert!(pretty_xml("<root></nope>").is_none());
        assert!(pretty_xml("not xml").is_none());
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
