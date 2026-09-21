/// 登録した本文から自動で付けるタグ。
pub fn auto_tags(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    let mut tags = Vec::new();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        tags.push("url".to_string());
    }
    if trimmed.contains('\n') {
        tags.push("multi".to_string());
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
}

pub fn expand_template(text: &str, ctx: &Expand) -> String {
    expand_seen(text, ctx, &mut std::collections::HashSet::new())
}

fn expand_seen(
    text: &str,
    ctx: &Expand,
    seen: &mut std::collections::HashSet<String>,
) -> String {
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

fn find_close(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == '}' && chars[i + 1] == '}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn token_value(
    inner: &str,
    ctx: &Expand,
    seen: &mut std::collections::HashSet<String>,
) -> Option<String> {
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
    if let Some(name) = inner.strip_prefix("ask:") {
        return Some(
            ctx.answers
                .get(name.trim())
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
    if let Some(spec) = inner.strip_prefix("pick:") {
        return Some(
            ctx.answers
                .get(&format!("pick:{}", spec.trim()))
                .cloned()
                .unwrap_or_default(),
        );
    }
    if let Some(name) = inner.strip_prefix('@') {
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
    if let Some(name) = inner.strip_prefix("env:") {
        let name = name.trim();
        if name.is_empty() {
            return Some(String::new());
        }
        return Some(std::env::var(name).unwrap_or_default());
    }
    if inner == "n" {
        return Some(ctx.n.to_string());
    }
    if let Some(width) = inner.strip_prefix("n:") {
        let width: usize = width.parse().ok()?;
        return Some(format!("{:0width$}", ctx.n, width = width.min(8)));
    }
    if let Some(fmt) = inner.strip_prefix("date:") {
        if fmt.is_empty() {
            return None;
        }
        return Some(ctx.now.format(fmt).to_string());
    }
    None
}

pub fn has_sel_token(text: &str) -> bool {
    walk_tokens(text, |inner| inner == "sel")
}

/// 出現順。同じ名前は 1 回だけ。
#[cfg_attr(not(test), allow(dead_code))]
pub fn ask_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    walk_tokens(text, |inner| {
        if let Some(name) = inner.strip_prefix("ask:") {
            let name = name.trim();
            if !names.iter().any(|entry| entry == name) {
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
        if let Some(raw) = inner.strip_prefix("pick:") {
            let spec = raw.trim().to_string();
            if !specs.iter().any(|(existing, _)| existing == &spec) {
                let options = spec
                    .split(',')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect();
                specs.push((spec, options));
            }
        }
        false
    });
    specs
}

pub fn prefix_lines(text: &str, prefix: &str, already: &str) -> String {
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

/// Shift+Enter 用の整形。行頭の引用符号を外し、余分な空行と前後の空白を落とす。
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
        assert_eq!(auto_tags("a\nb"), vec!["multi"]);
        assert_eq!(auto_tags(r"C:\work\memo.txt"), vec!["path"]);
        assert_eq!(auto_tags("/home/me/.bashrc"), vec!["path"]);
        assert!(auto_tags("ただの文章").is_empty());
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
        assert_eq!(expand_template("{{user}}@{{host}}", &ctx), "hatamon@pc");
    }

    #[test]
    fn expands_strftime_date() {
        let ctx = sample_ctx();
        assert_eq!(
            expand_template("{{date:%Y}}", &ctx),
            ctx.now.format("%Y").to_string()
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
        assert_eq!(expand_template("{{ask:missing}}", &ctx), "");
        assert!(has_sel_token("x {{sel}} y"));
        assert!(!has_sel_token("{{clip}}"));
        assert_eq!(ask_names("{{ask:a}} {{ask:b}} {{ask:a}}"), vec!["a", "b"]);
    }

    #[test]
    fn expands_app_front_and_pick() {
        let ctx = sample_ctx();
        assert_eq!(expand_template("{{app}}/{{front}}", &ctx), "code/TODO.md");
        assert_eq!(expand_template("{{pick:prod, stg}}", &ctx), "stg");
        assert_eq!(
            pick_specs("{{pick: a, b}} {{pick: a, b}}"),
            vec![("a, b".into(), vec!["a".into(), "b".into()])]
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
    fn prefixes_lines_unless_already_marked() {
        assert_eq!(prefix_lines("a\nb", "> ", ">"), "> a\n> b");
        assert_eq!(prefix_lines("> a\nb", "> ", ">"), "> a\n> b");
        assert_eq!(prefix_lines("a\n* b", "* ", "* "), "* a\n* b");
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
