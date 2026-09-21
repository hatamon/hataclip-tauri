/// `{{type:<Tab>}}` の中身。文字はそのまま、`<Key>` はキー。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeAtom {
    Text(String),
    Key(TypeStep),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeStep {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: TypeKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeKey {
    Char(char),
    Tab,
    Enter,
    Escape,
    Space,
    Backspace,
    Delete,
    Insert,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

pub fn parse_type_script(script: &str) -> Vec<TypeAtom> {
    let chars: Vec<char> = script.chars().collect();
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            if let Some(rel) = chars[i + 1..].iter().position(|&ch| ch == '>') {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                if let Some(step) = parse_angle(&inner) {
                    if !buf.is_empty() {
                        out.push(TypeAtom::Text(std::mem::take(&mut buf)));
                    }
                    out.push(TypeAtom::Key(step));
                    i = i + 1 + rel + 1;
                    continue;
                }
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        out.push(TypeAtom::Text(buf));
    }
    out
}

fn parse_angle(inner: &str) -> Option<TypeStep> {
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut key = None;
    for part in inner.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        let lower = part.to_ascii_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => ctrl = true,
            "shift" => shift = true,
            "alt" => alt = true,
            "tab" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Tab);
            }
            "enter" | "cr" | "return" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Enter);
            }
            "esc" | "escape" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Escape);
            }
            "space" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Space);
            }
            "bs" | "backspace" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Backspace);
            }
            "del" | "delete" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Delete);
            }
            "ins" | "insert" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Insert);
            }
            "up" | "uparrow" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Up);
            }
            "down" | "downarrow" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Down);
            }
            "left" | "leftarrow" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Left);
            }
            "right" | "rightarrow" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Right);
            }
            "home" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::Home);
            }
            "end" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::End);
            }
            "pageup" | "pgup" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::PageUp);
            }
            "pagedown" | "pgdn" | "pgdown" => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::PageDown);
            }
            fn_key if parse_fn_key(fn_key).is_some() => {
                if key.is_some() {
                    return None;
                }
                key = Some(TypeKey::F(parse_fn_key(fn_key)?));
            }
            one if one.chars().count() == 1 => {
                if key.is_some() {
                    return None;
                }
                let ch = one.chars().next()?;
                key = Some(TypeKey::Char(if ch.is_ascii_alphabetic() {
                    ch.to_ascii_lowercase()
                } else {
                    ch
                }));
            }
            _ => return None,
        }
    }
    Some(TypeStep {
        ctrl,
        shift,
        alt,
        key: key?,
    })
}

fn parse_fn_key(name: &str) -> Option<u8> {
    let digits = name.strip_prefix('f')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u8 = digits.parse().ok()?;
    if (1..=24).contains(&n) {
        Some(n)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: TypeKey) -> TypeAtom {
        TypeAtom::Key(TypeStep {
            ctrl: false,
            shift: false,
            alt: false,
            key,
        })
    }

    fn ctrl(ch: char) -> TypeAtom {
        TypeAtom::Key(TypeStep {
            ctrl: true,
            shift: false,
            alt: false,
            key: TypeKey::Char(ch),
        })
    }

    #[test]
    fn parses_tab_and_mixed_script() {
        assert_eq!(parse_type_script("<Tab>"), vec![key(TypeKey::Tab)]);
        assert_eq!(
            parse_type_script("abc<Tab>def"),
            vec![
                TypeAtom::Text("abc".into()),
                key(TypeKey::Tab),
                TypeAtom::Text("def".into()),
            ]
        );
        assert_eq!(
            parse_type_script("<Ctrl+A>abc<Tab><Ctrl+A>def<Enter>"),
            vec![
                ctrl('a'),
                TypeAtom::Text("abc".into()),
                key(TypeKey::Tab),
                ctrl('a'),
                TypeAtom::Text("def".into()),
                key(TypeKey::Enter),
            ]
        );
    }

    #[test]
    fn parses_named_keys_and_modifiers() {
        assert_eq!(parse_type_script("<CR>"), vec![key(TypeKey::Enter)]);
        assert_eq!(parse_type_script("<Esc>"), vec![key(TypeKey::Escape)]);
        assert_eq!(
            parse_type_script("<Shift+Tab>"),
            vec![TypeAtom::Key(TypeStep {
                ctrl: false,
                shift: true,
                alt: false,
                key: TypeKey::Tab,
            })]
        );
        assert_eq!(
            parse_type_script("<Ctrl+Shift+A>"),
            vec![TypeAtom::Key(TypeStep {
                ctrl: true,
                shift: true,
                alt: false,
                key: TypeKey::Char('a'),
            })]
        );
        assert_eq!(parse_type_script("<Up>"), vec![key(TypeKey::Up)]);
        assert_eq!(parse_type_script("<Down>"), vec![key(TypeKey::Down)]);
        assert_eq!(parse_type_script("<Left>"), vec![key(TypeKey::Left)]);
        assert_eq!(parse_type_script("<Right>"), vec![key(TypeKey::Right)]);
        assert_eq!(parse_type_script("<F1>"), vec![key(TypeKey::F(1))]);
        assert_eq!(parse_type_script("<F12>"), vec![key(TypeKey::F(12))]);
        assert_eq!(
            parse_type_script("<Ctrl+Shift+F1>"),
            vec![TypeAtom::Key(TypeStep {
                ctrl: true,
                shift: true,
                alt: false,
                key: TypeKey::F(1),
            })]
        );
    }

    #[test]
    fn unknown_brackets_stay_text() {
        assert_eq!(
            parse_type_script("<Nope>x"),
            vec![TypeAtom::Text("<Nope>x".into())]
        );
        assert_eq!(parse_type_script("<F0>"), vec![TypeAtom::Text("<F0>".into())]);
        assert_eq!(parse_type_script("<F25>"), vec![TypeAtom::Text("<F25>".into())]);
        assert_eq!(parse_type_script("<Ctrl>"), vec![TypeAtom::Text("<Ctrl>".into())]);
        assert_eq!(parse_type_script("id"), vec![TypeAtom::Text("id".into())]);
        assert!(parse_type_script("").is_empty());
    }
}
