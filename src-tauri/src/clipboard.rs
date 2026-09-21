pub fn read_clipboard_text() -> Option<String> {
    let text = peek_text()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

/// テキストとして読めた中身。画像などなら None。空文字は Some("")。
pub fn peek_text() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

pub fn write_clipboard_text(text: &str) -> bool {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text))
        .is_ok()
}
