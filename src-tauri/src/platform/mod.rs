use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{anchor_position, capture_foreground, restore_foreground, Foreground};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{anchor_position, capture_foreground, restore_foreground, Foreground};

#[cfg(not(any(windows, target_os = "linux")))]
mod unsupported;
#[cfg(not(any(windows, target_os = "linux")))]
pub use unsupported::{anchor_position, capture_foreground, restore_foreground, Foreground};

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
