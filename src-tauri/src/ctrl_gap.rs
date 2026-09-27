//! 行頭選択で Ctrl を一度離したあと、指がまだ押している Ctrl 付きのキーを受け直す。
//! 注入した Ctrl の上げ下げはホットキーに届かないので、物理キーだけを見る。

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Register,
    Show,
    Expand,
    Complete,
    Ranked(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chord {
    pub shift: bool,
    pub alt: bool,
    pub vk: u16,
    pub action: Action,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEv {
    pub vk: u16,
    pub down: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Pass,
    Swallow,
    Fire(Action),
}

const VK_CONTROL: u16 = 0x11;
const VK_LCONTROL: u16 = 0xA2;
const VK_RCONTROL: u16 = 0xA3;

pub struct Gap {
    pub chords: Vec<Chord>,
    armed: bool,
    left: bool,
    right: bool,
    fired: HashSet<u16>,
}

impl Gap {
    pub fn new() -> Self {
        Self {
            chords: Vec::new(),
            armed: false,
            left: false,
            right: false,
            fired: HashSet::new(),
        }
    }

    pub fn physical(&self) -> bool {
        self.left || self.right
    }

    pub fn arm(&mut self) {
        if self.physical() {
            self.armed = true;
        }
    }

    pub fn disarm(&mut self) {
        self.armed = false;
        self.fired.clear();
    }

    pub fn on_key(&mut self, ev: KeyEv) -> Decision {
        if is_ctrl(ev.vk) {
            let down = ev.down;
            if ev.vk == VK_RCONTROL {
                self.right = down;
            } else {
                self.left = down;
            }
            if !self.physical() {
                self.disarm();
            }
            return Decision::Pass;
        }
        if !ev.down {
            if self.fired.remove(&ev.vk) {
                return Decision::Swallow;
            }
            return Decision::Pass;
        }
        if !self.armed || self.fired.contains(&ev.vk) {
            if self.armed && self.fired.contains(&ev.vk) {
                return Decision::Swallow;
            }
            return Decision::Pass;
        }
        let Some(chord) = self
            .chords
            .iter()
            .find(|chord| chord.vk == ev.vk && chord.shift == ev.shift && chord.alt == ev.alt)
        else {
            return Decision::Pass;
        };
        self.fired.insert(ev.vk);
        Decision::Fire(chord.action.clone())
    }
}

impl Default for Gap {
    fn default() -> Self {
        Self::new()
    }
}

fn is_ctrl(vk: u16) -> bool {
    vk == VK_CONTROL || vk == VK_LCONTROL || vk == VK_RCONTROL
}

/// `Control+Digit8` のように Ctrl を含むショートカットだけ。Ctrl が無ければなし。
pub fn parse_ctrl_shortcut(spec: &str) -> Option<(bool, bool, u16)> {
    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut vk = None;
    for part in spec.split('+') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        match part {
            "Control" | "Ctrl" => ctrl = true,
            "Shift" => shift = true,
            "Alt" => alt = true,
            "Super" | "Meta" | "Win" => return None,
            other => {
                if vk.is_some() {
                    return None;
                }
                vk = Some(vk_of(other)?);
            }
        }
    }
    if !ctrl {
        return None;
    }
    Some((shift, alt, vk?))
}

fn vk_of(name: &str) -> Option<u16> {
    if let Some(rest) = name.strip_prefix("Digit") {
        let n: u16 = rest.parse().ok()?;
        if n <= 9 {
            return Some(0x30 + n);
        }
        return None;
    }
    if let Some(rest) = name.strip_prefix("Numpad") {
        if let Ok(n) = rest.parse::<u16>() {
            if n <= 9 {
                return Some(0x60 + n);
            }
        }
        return Some(match rest {
            "Multiply" => 0x6A,
            "Add" => 0x6B,
            "Subtract" => 0x6D,
            "Decimal" => 0x6E,
            "Divide" => 0x6F,
            "Enter" => 0x0D,
            _ => return None,
        });
    }
    if let Some(rest) = name.strip_prefix("Key") {
        let mut chars = rest.chars();
        let ch = chars.next()?;
        if chars.next().is_none() && ch.is_ascii_alphabetic() {
            return Some(ch.to_ascii_uppercase() as u16);
        }
        return None;
    }
    if let Some(rest) = name.strip_prefix('F') {
        if rest.chars().all(|ch| ch.is_ascii_digit()) {
            let n: u16 = rest.parse().ok()?;
            if (1..=24).contains(&n) {
                return Some(0x6F + n);
            }
        }
    }
    Some(match name {
        "Backspace" => 0x08,
        "Tab" => 0x09,
        "Enter" | "Return" => 0x0D,
        "Escape" => 0x1B,
        "Space" => 0x20,
        "PageUp" => 0x21,
        "PageDown" => 0x22,
        "End" => 0x23,
        "Home" => 0x24,
        "ArrowLeft" | "Left" => 0x25,
        "ArrowUp" | "Up" => 0x26,
        "ArrowRight" | "Right" => 0x27,
        "ArrowDown" | "Down" => 0x28,
        "Insert" => 0x2D,
        "Delete" => 0x2E,
        "Minus" => 0xBD,
        "Equal" => 0xBB,
        "Comma" => 0xBC,
        "Period" => 0xBE,
        "Slash" => 0xBF,
        "Backquote" => 0xC0,
        "BracketLeft" => 0xDB,
        "Backslash" => 0xDC,
        "BracketRight" => 0xDD,
        "Quote" => 0xDE,
        "Semicolon" => 0xBA,
        _ => return None,
    })
}

static GAP: std::sync::LazyLock<Mutex<Gap>> = std::sync::LazyLock::new(|| Mutex::new(Gap::new()));

static HANDLER: Mutex<Option<Arc<dyn Fn(Action) + Send + Sync>>> = Mutex::new(None);

static TX: Mutex<Option<std::sync::mpsc::Sender<Action>>> = Mutex::new(None);

fn gap() -> std::sync::MutexGuard<'static, Gap> {
    GAP.lock().unwrap_or_else(|err| err.into_inner())
}

pub fn publish(chords: Vec<Chord>, handler: Arc<dyn Fn(Action) + Send + Sync>) {
    ensure_worker();
    gap().chords = chords;
    *HANDLER.lock().unwrap_or_else(|err| err.into_inner()) = Some(handler);
}

pub fn clear() {
    let mut gap = gap();
    gap.chords.clear();
    gap.disarm();
}

pub fn physical() -> bool {
    gap().physical()
}

pub fn arm() {
    gap().arm();
}

pub fn disarm() {
    gap().disarm();
}

pub fn on_hook_key(vk: u16, down: bool, shift: bool, alt: bool) -> bool {
    let decision = gap().on_key(KeyEv {
        vk,
        down,
        shift,
        alt,
    });
    match decision {
        Decision::Pass => false,
        Decision::Swallow => true,
        Decision::Fire(action) => {
            if let Some(tx) = TX.lock().unwrap_or_else(|err| err.into_inner()).as_ref() {
                let _ = tx.send(action);
            }
            true
        }
    }
}

pub fn ensure_worker() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        *TX.lock().unwrap_or_else(|err| err.into_inner()) = Some(tx);
        std::thread::spawn(move || {
            while let Ok(action) = rx.recv() {
                let handler = HANDLER
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .clone();
                if let Some(handler) = handler {
                    handler(action);
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expand_gap() -> Gap {
        let mut gap = Gap::new();
        gap.chords = vec![
            Chord {
                shift: false,
                alt: false,
                vk: 0x38,
                action: Action::Expand,
            },
            Chord {
                shift: false,
                alt: false,
                vk: 0x37,
                action: Action::Show,
            },
            Chord {
                shift: true,
                alt: false,
                vk: 0x31,
                action: Action::Ranked(0),
            },
        ];
        gap
    }

    fn down(vk: u16, shift: bool) -> KeyEv {
        KeyEv {
            vk,
            down: true,
            shift,
            alt: false,
        }
    }

    #[test]
    fn armed_digit_fires_once_and_later_presses_are_swallowed() {
        let mut gap = expand_gap();
        assert_eq!(gap.on_key(down(0xA2, false)), Decision::Pass);
        gap.arm();
        assert_eq!(
            gap.on_key(down(0x38, false)),
            Decision::Fire(Action::Expand)
        );
        assert_eq!(gap.on_key(down(0x38, false)), Decision::Swallow);
        assert_eq!(gap.on_key(down(0x37, false)), Decision::Fire(Action::Show));
    }

    #[test]
    fn releasing_ctrl_lets_the_digit_through() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2, false));
        gap.arm();
        gap.on_key(KeyEv {
            vk: 0xA2,
            down: false,
            shift: false,
            alt: false,
        });
        assert!(!gap.physical());
        assert_eq!(gap.on_key(down(0x38, false)), Decision::Pass);
    }

    #[test]
    fn arm_does_nothing_until_ctrl_is_physically_down() {
        let mut gap = expand_gap();
        gap.arm();
        assert_eq!(gap.on_key(down(0x38, false)), Decision::Pass);
    }

    #[test]
    fn shift_digit_is_the_shifted_chord() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2, false));
        gap.arm();
        assert_eq!(
            gap.on_key(down(0x31, true)),
            Decision::Fire(Action::Ranked(0))
        );
        gap.on_key(KeyEv {
            vk: 0x31,
            down: false,
            shift: true,
            alt: false,
        });
        assert_eq!(gap.on_key(down(0x31, false)), Decision::Pass);
    }

    #[test]
    fn parse_keeps_only_ctrl_chords() {
        assert_eq!(
            parse_ctrl_shortcut("Control+Digit8"),
            Some((false, false, 0x38))
        );
        assert_eq!(
            parse_ctrl_shortcut("Control+Shift+Digit1"),
            Some((true, false, 0x31))
        );
        assert_eq!(
            parse_ctrl_shortcut("Control+Shift+KeyH"),
            Some((true, false, b'H' as u16))
        );
        assert!(parse_ctrl_shortcut("Shift+Digit8").is_none());
        assert!(parse_ctrl_shortcut("Control+Super+Digit8").is_none());
    }
}
