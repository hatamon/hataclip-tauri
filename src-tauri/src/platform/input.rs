use crate::chord::{self, Chord, ChordKey};
use crate::keys::{TypeAtom, TypeKey, TypeStep};
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::time::{Duration, Instant};

pub fn simulate_paste(spec: &str) -> bool {
    simulate_chord(spec)
}

pub fn simulate_copy(spec: &str) -> bool {
    simulate_chord(spec)
}

pub fn simulate_chord(spec: &str) -> bool {
    let Some(chord) = chord::parse(spec) else {
        return false;
    };
    send_chord(chord)
}

fn modifiers_held() -> (bool, bool) {
    super::windows::modifiers_held()
}

/// 本文を 1 文字ずつ前面へ送る。2 秒を超えたら中止。
pub fn simulate_type(text: &str) -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    let started = Instant::now();
    for ch in text.chars() {
        if started.elapsed() >= Duration::from_secs(2) {
            return false;
        }
        let ok = if ch == '\n' || ch == '\r' {
            enigo.key(Key::Return, Click).is_ok()
        } else if ch == '\t' {
            enigo.key(Key::Tab, Click).is_ok()
        } else {
            enigo.text(&ch.to_string()).is_ok()
        };
        if !ok {
            return false;
        }
    }
    true
}

fn send_chord(chord: Chord) -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    let (ctrl_held, shift_held) = modifiers_held();
    let press_ctrl = chord.ctrl && !ctrl_held;
    let press_shift = chord.shift && !shift_held;
    let mut ok = true;
    if press_ctrl {
        ok &= enigo.key(Key::Control, Press).is_ok();
    }
    if press_shift {
        ok &= enigo.key(Key::Shift, Press).is_ok();
    }
    let key = match chord.key {
        ChordKey::Char(ch) => Key::Unicode(ch),
        ChordKey::Insert => Key::Insert,
        ChordKey::Home => Key::Home,
    };
    ok &= enigo.key(key, Click).is_ok();
    if press_shift {
        ok &= enigo.key(Key::Shift, Release).is_ok();
    }
    if press_ctrl {
        ok &= enigo.key(Key::Control, Release).is_ok();
    }
    ok
}

/// `{{type:}}` の断片を前面へ送る。
pub fn simulate_type_atoms(atoms: &[TypeAtom]) -> bool {
    for (index, atom) in atoms.iter().enumerate() {
        let ok = match atom {
            TypeAtom::Text(text) => simulate_type(text),
            TypeAtom::Key(step) => send_type_step(step),
        };
        if !ok {
            return false;
        }
        if index + 1 < atoms.len() {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    true
}

fn send_type_step(step: &TypeStep) -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    let (ctrl_held, shift_held) = modifiers_held();
    let press_ctrl = step.ctrl && !ctrl_held;
    let press_shift = step.shift && !shift_held;
    let mut ok = true;
    if press_ctrl {
        ok &= enigo.key(Key::Control, Press).is_ok();
    }
    if press_shift {
        ok &= enigo.key(Key::Shift, Press).is_ok();
    }
    if step.alt {
        ok &= enigo.key(Key::Alt, Press).is_ok();
    }
    ok &= enigo.key(enigo_key(step.key), Click).is_ok();
    if step.alt {
        ok &= enigo.key(Key::Alt, Release).is_ok();
    }
    if press_shift {
        ok &= enigo.key(Key::Shift, Release).is_ok();
    }
    if press_ctrl {
        ok &= enigo.key(Key::Control, Release).is_ok();
    }
    ok
}

fn enigo_key(key: TypeKey) -> Key {
    match key {
        TypeKey::Char(ch) => Key::Unicode(ch),
        TypeKey::Tab => Key::Tab,
        TypeKey::Enter => Key::Return,
        TypeKey::Escape => Key::Escape,
        TypeKey::Space => Key::Space,
        TypeKey::Backspace => Key::Backspace,
        TypeKey::Delete => Key::Delete,
        TypeKey::Insert => Key::Insert,
        TypeKey::Up => Key::UpArrow,
        TypeKey::Down => Key::DownArrow,
        TypeKey::Left => Key::LeftArrow,
        TypeKey::Right => Key::RightArrow,
        TypeKey::Home => Key::Home,
        TypeKey::End => Key::End,
        TypeKey::PageUp => Key::PageUp,
        TypeKey::PageDown => Key::PageDown,
        TypeKey::F(n) => enigo_fn_key(n),
    }
}

fn enigo_fn_key(n: u8) -> Key {
    match n {
        1 => Key::F1,
        2 => Key::F2,
        3 => Key::F3,
        4 => Key::F4,
        5 => Key::F5,
        6 => Key::F6,
        7 => Key::F7,
        8 => Key::F8,
        9 => Key::F9,
        10 => Key::F10,
        11 => Key::F11,
        12 => Key::F12,
        13 => Key::F13,
        14 => Key::F14,
        15 => Key::F15,
        16 => Key::F16,
        17 => Key::F17,
        18 => Key::F18,
        19 => Key::F19,
        20 => Key::F20,
        21 => Key::F21,
        22 => Key::F22,
        23 => Key::F23,
        24 => Key::F24,
        _ => Key::F1,
    }
}
