use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{capture_foreground, context_key, restore_foreground, Foreground};

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::{capture_foreground, context_key, restore_foreground, Foreground};

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

fn chord(key: Key) -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    enigo.key(Key::Control, Press).is_ok()
        && enigo.key(key, Click).is_ok()
        && enigo.key(Key::Control, Release).is_ok()
}
