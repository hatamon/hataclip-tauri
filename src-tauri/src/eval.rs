use std::collections::HashMap;

/// `:sh` / `:echo` のように、貼る結果だけを出すコマンド。
pub fn colon_output(line: &str, vars: &HashMap<String, String>) -> Option<String> {
    let trimmed = line.trim();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_and_sh_only() {
        let mut vars = HashMap::new();
        vars.insert("a".into(), "2".into());
        assert_eq!(colon_output(":echo a+3", &vars).as_deref(), Some("5"));
        assert_eq!(colon_output("echo 2+3*4", &vars).as_deref(), Some("14"));
        assert_eq!(colon_output(":echo (2+3)*4", &vars).as_deref(), Some("20"));
        assert_eq!(colon_output(":echo", &vars), None);
        assert_eq!(colon_output(":clear yes", &vars), None);
        assert_eq!(colon_output("hello", &vars), None);
        assert_eq!(colon_output(":sh ", &vars), None);
    }
}
