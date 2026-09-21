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

pub fn simulate_paste() -> bool {
    chord(Key::Unicode('v'))
}

pub fn simulate_copy() -> bool {
    chord(Key::Unicode('c'))
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

fn chord(key: Key) -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    enigo.key(Key::Control, Press).is_ok()
        && enigo.key(key, Click).is_ok()
        && enigo.key(Key::Control, Release).is_ok()
}
