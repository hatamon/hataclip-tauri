use crate::keys::TypeAtom;
use std::process::Command;

#[derive(Clone, Copy)]
pub struct Foreground;

pub fn capture_foreground() -> Option<Foreground> {
    None
}

pub fn restore_foreground(_fg: &Foreground) -> bool {
    false
}

pub fn context_key(_fg: &Foreground) -> Option<String> {
    None
}

pub fn app_and_title(_fg: &Foreground) -> (String, String) {
    (String::new(), String::new())
}

#[allow(dead_code)]
pub fn simulate_paste(_spec: &str) -> bool {
    false
}

#[allow(dead_code)]
pub fn simulate_copy(_spec: &str) -> bool {
    false
}

#[allow(dead_code)]
pub fn simulate_type(_text: &str) -> bool {
    false
}

#[allow(dead_code)]
pub fn simulate_type_atoms(_atoms: &[TypeAtom]) -> bool {
    false
}

pub fn open_url(url: &str) -> bool {
    spawn_xdg(url)
}

pub fn open_file(text: &str) -> bool {
    spawn_xdg(text)
}

pub fn open_dir(text: &str) -> bool {
    spawn_xdg(text)
}

pub fn open_argv(target: &str) -> Vec<String> {
    vec!["xdg-open".to_string(), target.to_string()]
}

fn spawn_xdg(target: &str) -> bool {
    let argv = open_argv(target);
    Command::new(&argv[0]).args(&argv[1..]).spawn().is_ok()
}

#[cfg(test)]
mod tests {
    use super::{open_argv, simulate_copy, simulate_paste, simulate_type, simulate_type_atoms};

    #[test]
    fn xdg_open_keeps_the_target() {
        assert_eq!(
            open_argv("https://example.com"),
            ["xdg-open", "https://example.com"]
        );
        assert_eq!(open_argv("/tmp"), ["xdg-open", "/tmp"]);
        assert_eq!(open_argv("/tmp/notes.md"), ["xdg-open", "/tmp/notes.md"]);
    }

    #[test]
    fn key_injection_is_off() {
        assert!(!simulate_paste("ctrl+v"));
        assert!(!simulate_copy("ctrl+c"));
        assert!(!simulate_type("hi"));
        assert!(!simulate_type_atoms(&[]));
    }
}
