use crate::chord::{self, Chord, ChordKey};
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::time::{Duration, Instant};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{app_and_title, capture_foreground, context_key, restore_foreground, Foreground};

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::{app_and_title, capture_foreground, context_key, restore_foreground, Foreground};

#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

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
    let mut ok = true;
    if chord.ctrl {
        ok &= enigo.key(Key::Control, Press).is_ok();
    }
    if chord.shift {
        ok &= enigo.key(Key::Shift, Press).is_ok();
    }
    let key = match chord.key {
        ChordKey::Char(ch) => Key::Unicode(ch),
        ChordKey::Insert => Key::Insert,
    };
    ok &= enigo.key(key, Click).is_ok();
    if chord.shift {
        ok &= enigo.key(Key::Shift, Release).is_ok();
    }
    if chord.ctrl {
        ok &= enigo.key(Key::Control, Release).is_ok();
    }
    ok
}
