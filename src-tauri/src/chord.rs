/// 前面アプリへ送るコピー／貼り付け。`ctrl+c` や `shift+insert`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub ctrl: bool,
    pub shift: bool,
    pub key: ChordKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordKey {
    Char(char),
    Insert,
    Home,
}

pub const DEFAULT_COPY: &str = "ctrl+c";
pub const DEFAULT_PASTE: &str = "ctrl+v";

pub fn parse(input: &str) -> Option<Chord> {
    let mut ctrl = false;
    let mut shift = false;
    let mut key = None;
    for part in input.split('+') {
        let part = part.trim().to_ascii_lowercase();
        if part.is_empty() {
            return None;
        }
        match part.as_str() {
            "ctrl" | "control" => ctrl = true,
            "shift" => shift = true,
            "insert" | "ins" => {
                if key.is_some() {
                    return None;
                }
                key = Some(ChordKey::Insert);
            }
            "home" => {
                if key.is_some() {
                    return None;
                }
                key = Some(ChordKey::Home);
            }
            one if one.chars().count() == 1 => {
                let ch = one.chars().next()?;
                if !ch.is_ascii_alphabetic() {
                    return None;
                }
                if key.is_some() {
                    return None;
                }
                key = Some(ChordKey::Char(ch));
            }
            _ => return None,
        }
    }
    if !ctrl && !shift {
        return None;
    }
    Some(Chord { ctrl, shift, key: key? })
}

pub fn display(chord: &Chord) -> String {
    let mut parts = Vec::new();
    if chord.ctrl {
        parts.push("ctrl".to_string());
    }
    if chord.shift {
        parts.push("shift".to_string());
    }
    parts.push(match chord.key {
        ChordKey::Char(ch) => ch.to_string(),
        ChordKey::Insert => "insert".to_string(),
        ChordKey::Home => "home".to_string(),
    });
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_copy_paste_and_insert() {
        assert_eq!(
            parse("ctrl+c"),
            Some(Chord {
                ctrl: true,
                shift: false,
                key: ChordKey::Char('c'),
            })
        );
        assert_eq!(
            parse("Shift+Insert"),
            Some(Chord {
                ctrl: false,
                shift: true,
                key: ChordKey::Insert,
            })
        );
        assert_eq!(
            parse("ctrl+shift+v"),
            Some(Chord {
                ctrl: true,
                shift: true,
                key: ChordKey::Char('v'),
            })
        );
        assert_eq!(display(&parse("CONTROL+INS").unwrap()), "ctrl+insert");
        assert_eq!(display(&parse("ctrl+shift+v").unwrap()), "ctrl+shift+v");
    }

    #[test]
    fn rejects_unknown_or_bare_keys() {
        assert!(parse("v").is_none());
        assert!(parse("ctrl+").is_none());
        assert!(parse("alt+v").is_none());
        assert!(parse("ctrl+nope").is_none());
        assert!(parse("").is_none());
    }
}
