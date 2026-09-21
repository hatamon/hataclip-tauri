use std::collections::HashMap;

/// `:sh` のように、貼る結果だけを出すコマンド。`:echo` は expr 側。
pub fn colon_output(line: &str, vars: &HashMap<String, String>) -> Option<String> {
    let _ = vars;
    let trimmed = line.trim();
    let body = trimmed.strip_prefix(':').unwrap_or(trimmed).trim();
    if let Some(script) = body.strip_prefix("sh ") {
        let script = script.trim();
        if script.is_empty() {
            return None;
        }
        return crate::shell::run_script(script).ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_sh_produces_output() {
        let vars = HashMap::new();
        assert_eq!(colon_output(":clear yes", &vars), None);
        assert_eq!(colon_output("hello", &vars), None);
        assert_eq!(colon_output(":sh ", &vars), None);
    }
}
