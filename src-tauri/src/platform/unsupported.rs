#[derive(Clone, Copy)]
pub struct Foreground;

pub fn capture_foreground() -> Option<Foreground> {
    None
}

pub fn restore_foreground(_fg: &Foreground) -> bool {
    false
}
