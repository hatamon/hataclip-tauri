use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{capture_foreground, restore_foreground, Foreground};

#[cfg(not(windows))]
mod unsupported;
#[cfg(not(windows))]
pub use unsupported::{capture_foreground, restore_foreground, Foreground};

#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn simulate_paste() -> bool {
    let mut enigo = match Enigo::new(&Settings::default()) {
        Ok(enigo) => enigo,
        Err(_) => return false,
    };
    enigo.key(Key::Control, Press).is_ok()
        && enigo.key(Key::Unicode('v'), Click).is_ok()
        && enigo.key(Key::Control, Release).is_ok()
}
