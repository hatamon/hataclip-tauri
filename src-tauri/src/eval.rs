use crate::text;
use std::collections::HashMap;

/// `:sh` / `:echo` のように、貼る結果だけを出すコマンド。
pub fn colon_output(line: &str, vars: &HashMap<String, String>) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(':') {
        return None;
    }
    let body = trimmed.strip_prefix(':').unwrap_or(trimmed).trim();
    if let Some(script) = body.strip_prefix("sh ") {
        let script = script.trim();
        if script.is_empty() {
            return None;
        }
        return crate::shell::run_script(script).ok();
    }
    if let Some(expr) = body.strip_prefix("echo ") {
        let expr = expr.trim();
        if expr.is_empty() {
            return None;
        }
        return crate::expr::eval_with(expr, vars).map(crate::expr::format_number);
    }
    None
}

/// 前面で選んだ `{{date}}` / `:sh` / `:echo` を置き換える。
pub fn resolve_selection(
    text: &str,
    ctx: &text::Expand,
    last_sh: Option<&str>,
) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(out) = colon_line(trimmed, &ctx.vars, last_sh) {
        return Some(out);
    }
    if !trimmed.contains("{{") {
        return None;
    }
    let expanded = text::expand_template(trimmed, ctx);
    if let Some(out) = colon_line(expanded.trim(), &ctx.vars, last_sh) {
        return Some(out);
    }
    if expanded != trimmed || text::has_type_token(trimmed) {
        Some(expanded)
    } else {
        None
    }
}

fn colon_line(
    line: &str,
    vars: &HashMap<String, String>,
    last_sh: Option<&str>,
) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(':') {
        return None;
    }
    let body = trimmed.strip_prefix(':').unwrap_or(trimmed).trim();
    if body == "@" {
        return last_sh.and_then(|script| crate::shell::run_script(script).ok());
    }
    colon_output(trimmed, vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> text::Expand {
        text::Expand {
            date: "2026/09/21".into(),
            time: "13:00".into(),
            clip: String::new(),
            sel: String::new(),
            n: 1,
            uuid: "u".into(),
            user: "hatamon".into(),
            host: "pc".into(),
            app: String::new(),
            front: String::new(),
            focus: String::new(),
            now: chrono::Local::now(),
            answers: HashMap::new(),
            aliases: HashMap::new(),
            vars: HashMap::from([("a".into(), "{{date}}".into())]),
            tags: HashMap::new(),
        }
    }

    #[test]
    fn echo_and_sh_only() {
        let mut vars = HashMap::new();
        vars.insert("a".into(), "2".into());
        assert_eq!(colon_output(":echo a+3", &vars).as_deref(), Some("5"));
        assert_eq!(colon_output("echo 2+3*4", &vars), None);
        assert_eq!(colon_output(":echo 2+3*4", &vars).as_deref(), Some("14"));
        assert_eq!(colon_output(":echo (2+3)*4", &vars).as_deref(), Some("20"));
        assert_eq!(colon_output(":echo", &vars), None);
        assert_eq!(colon_output(":clear yes", &vars), None);
        assert_eq!(colon_output("hello", &vars), None);
        assert_eq!(colon_output(":sh ", &vars), None);
    }

    #[test]
    fn resolve_selection_expands_templates_and_echo() {
        let ctx = ctx();
        assert_eq!(
            resolve_selection("{{date}}", &ctx, None).as_deref(),
            Some("2026/09/21")
        );
        assert_eq!(
            resolve_selection("date {{var:a}}", &ctx, None).as_deref(),
            Some("date 2026/09/21")
        );
        assert_eq!(
            resolve_selection("date {{var a}}", &ctx, None).as_deref(),
            Some("date 2026/09/21")
        );
        assert_eq!(
            resolve_selection(":echo 2+3", &ctx, None).as_deref(),
            Some("5")
        );
        assert_eq!(resolve_selection("#foo", &ctx, None), None);
        assert_eq!(resolve_selection("hello", &ctx, None), None);
        assert_eq!(resolve_selection("{{nope}}", &ctx, None), None);
        assert_eq!(
            resolve_selection("{{type:<Tab>}}", &ctx, None).as_deref(),
            Some("{{type:<Tab>}}")
        );
    }
}

