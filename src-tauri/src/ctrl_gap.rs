//! 行頭選択で修飾キーを一度離したあと、指がまだ押している Ctrl 付きのキーを受け直す。
//! 注入した上げ下げは見ない。物理キーだけを見る。

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Register,
    Show,
    Expand,
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
const VK_SHIFT: u16 = 0x10;
const VK_LSHIFT: u16 = 0xA0;
const VK_RSHIFT: u16 = 0xA1;
const VK_MENU: u16 = 0x12;
const VK_LMENU: u16 = 0xA4;
const VK_RMENU: u16 = 0xA5;

pub struct Gap {
    pub chords: Vec<Chord>,
    armed: bool,
    left: bool,
    right: bool,
    shift_left: bool,
    shift_right: bool,
    alt_left: bool,
    alt_right: bool,
    fired: HashSet<u16>,
}

impl Gap {
    pub fn new() -> Self {
        Self {
            chords: Vec::new(),
            armed: false,
            left: false,
            right: false,
            shift_left: false,
            shift_right: false,
            alt_left: false,
            alt_right: false,
            fired: HashSet::new(),
        }
    }

    pub fn physical(&self) -> bool {
        self.left || self.right
    }

    pub fn physical_shift(&self) -> bool {
        self.shift_left || self.shift_right
    }

    pub fn physical_alt(&self) -> bool {
        self.alt_left || self.alt_right
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

    /// 指が Ctrl を離しているのに追跡だけ残っているとき、追跡を消す。消したら真。
    pub fn forget_ctrl_if_finger_up(&mut self, finger_down: bool) -> bool {
        if finger_down || !self.physical() {
            return false;
        }
        self.left = false;
        self.right = false;
        self.disarm();
        true
    }

    pub fn on_key(&mut self, ev: KeyEv) -> Decision {
        if is_ctrl(ev.vk) {
            if ev.vk == VK_RCONTROL {
                self.right = ev.down;
            } else {
                self.left = ev.down;
            }
            if !self.physical() {
                self.disarm();
            }
            return Decision::Pass;
        }
        if is_shift(ev.vk) {
            if ev.vk == VK_RSHIFT {
                self.shift_right = ev.down;
            } else {
                self.shift_left = ev.down;
            }
            return Decision::Pass;
        }
        if is_alt(ev.vk) {
            if ev.vk == VK_RMENU {
                self.alt_right = ev.down;
            } else {
                self.alt_left = ev.down;
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
        let shift = self.physical_shift();
        let alt = self.physical_alt();
        let Some(chord) = self
            .chords
            .iter()
            .find(|chord| chord.vk == ev.vk && chord.shift == shift && chord.alt == alt)
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

fn is_shift(vk: u16) -> bool {
    vk == VK_SHIFT || vk == VK_LSHIFT || vk == VK_RSHIFT
}

fn is_alt(vk: u16) -> bool {
    vk == VK_MENU || vk == VK_LMENU || vk == VK_RMENU
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

/// 左も右も離れていれば、注入で残った Ctrl の追跡を消す。消したら真。
pub fn sync_ctrl_with_finger() -> bool {
    gap().forget_ctrl_if_finger_up(finger_ctrl_down())
}

#[cfg(windows)]
fn finger_ctrl_down() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LCONTROL, VK_RCONTROL,
    };
    unsafe {
        (GetAsyncKeyState(i32::from(VK_LCONTROL)) as u16) & 0x8000 != 0
            || (GetAsyncKeyState(i32::from(VK_RCONTROL)) as u16) & 0x8000 != 0
    }
}

#[cfg(not(windows))]
fn finger_ctrl_down() -> bool {
    false
}

pub fn physical_shift() -> bool {
    gap().physical_shift()
}

pub fn physical_alt() -> bool {
    gap().physical_alt()
}

pub fn arm() {
    gap().arm();
}

pub fn disarm() {
    gap().disarm();
}

static SENDING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 自分の enigo 送信のあいだだけ真。フックがこの送信を「指のキー」と誤認しないため。
pub(crate) fn begin_send() {
    SENDING.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub(crate) fn end_send() {
    SENDING.store(false, std::sync::atomic::Ordering::SeqCst);
}

pub fn is_sending() -> bool {
    SENDING.load(std::sync::atomic::Ordering::SeqCst)
}

/// フックへ渡すキーを物理キーとして扱うかどうか。
/// 本番は注入キーを無視する。E2E（`test_mode`）は注入キーも受けるが、
/// 自分の送信中（`sending`）だけは、送った側を指のキーと誤認しないよう無視する。
pub fn should_treat_as_physical(injected: bool, test_mode: bool, sending: bool) -> bool {
    !injected || (test_mode && !sending)
}

pub fn on_hook_key(vk: u16, down: bool) -> bool {
    let decision = gap().on_key(KeyEv { vk, down });
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

    fn down(vk: u16) -> KeyEv {
        KeyEv { vk, down: true }
    }

    fn up(vk: u16) -> KeyEv {
        KeyEv { vk, down: false }
    }

    #[test]
    fn armed_digit_fires_once_and_later_presses_are_swallowed() {
        let mut gap = expand_gap();
        assert_eq!(gap.on_key(down(0xA2)), Decision::Pass);
        gap.arm();
        assert_eq!(gap.on_key(down(0x38)), Decision::Fire(Action::Expand));
        assert_eq!(gap.on_key(down(0x38)), Decision::Swallow);
        assert_eq!(gap.on_key(down(0x37)), Decision::Fire(Action::Show));
    }

    #[test]
    fn releasing_ctrl_lets_the_digit_through() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2));
        gap.arm();
        gap.on_key(up(0xA2));
        assert!(!gap.physical());
        assert_eq!(gap.on_key(down(0x38)), Decision::Pass);
    }

    #[test]
    fn finger_up_drops_a_stuck_ctrl_and_disarms() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2));
        gap.arm();
        assert!(gap.forget_ctrl_if_finger_up(false));
        assert!(!gap.physical());
        assert_eq!(gap.on_key(down(0x37)), Decision::Pass);
        assert!(!gap.forget_ctrl_if_finger_up(false));
    }

    #[test]
    fn finger_still_down_keeps_ctrl() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2));
        assert!(!gap.forget_ctrl_if_finger_up(true));
        assert!(gap.physical());
    }

    #[test]
    fn arm_does_nothing_until_ctrl_is_physically_down() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA0));
        gap.arm();
        assert_eq!(gap.on_key(down(0x31)), Decision::Pass);
        gap.on_key(up(0xA0));
        gap.arm();
        assert_eq!(gap.on_key(down(0x38)), Decision::Pass);
    }

    #[test]
    fn shift_digit_follows_the_physical_shift_key() {
        let mut gap = expand_gap();
        gap.on_key(down(0xA2));
        gap.on_key(down(0xA0));
        gap.arm();
        assert_eq!(gap.on_key(down(0x31)), Decision::Fire(Action::Ranked(0)));
        gap.on_key(up(0x31));
        assert_eq!(gap.on_key(down(0x31)), Decision::Fire(Action::Ranked(0)));
        gap.on_key(up(0x31));
        gap.on_key(up(0xA0));
        assert_eq!(gap.on_key(down(0x31)), Decision::Pass);
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

    #[test]
    fn physical_keys_are_always_accepted() {
        assert!(should_treat_as_physical(false, false, false));
        assert!(should_treat_as_physical(false, true, false));
        assert!(should_treat_as_physical(false, false, true));
    }

    #[test]
    fn injected_keys_are_rejected_outside_test_mode() {
        assert!(!should_treat_as_physical(true, false, false));
        assert!(!should_treat_as_physical(true, false, true));
    }

    #[test]
    fn injected_keys_are_accepted_in_test_mode_unless_we_are_sending() {
        assert!(should_treat_as_physical(true, true, false));
        assert!(!should_treat_as_physical(true, true, true));
    }
}
