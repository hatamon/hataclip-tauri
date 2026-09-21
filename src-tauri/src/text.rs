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

fn looks_like_path(text: &str) -> bool {
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
    pub n: u32,
    pub uuid: String,
    pub user: String,
    pub host: String,
    pub now: chrono::DateTime<chrono::Local>,
}

pub fn expand_template(text: &str, ctx: &Expand) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if let Some(value) = token_value(inner.trim(), ctx) {
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

fn token_value(inner: &str, ctx: &Expand) -> Option<String> {
    if inner == "date" {
        return Some(ctx.date.clone());
    }
    if inner == "time" {
        return Some(ctx.time.clone());
    }
    if inner == "clip" {
        return Some(ctx.clip.clone());
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
            n: 3,
            uuid: "uuid-here".into(),
            user: "hatamon".into(),
            host: "pc".into(),
            now: chrono::Local::now(),
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
