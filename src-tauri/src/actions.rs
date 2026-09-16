use tauri_plugin_global_shortcut::Shortcut;

pub fn parse(value: &str) -> Result<Shortcut, String> {
    let shortcut: Shortcut = value
        .parse()
        .map_err(|_| format!("使えないキー: {value}"))?;
    if shortcut.mods.is_empty() {
        return Err(format!("修飾キーが要る: {value}"));
    }
    Ok(shortcut)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifier_combinations() {
        assert!(parse("Control+Digit4").is_ok());
        assert!(parse("Control+Shift+KeyY").is_ok());
    }

    #[test]
    fn rejects_shortcut_without_modifier() {
        assert!(parse("Digit4").is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(parse("Control+Nope").is_err());
    }
}
