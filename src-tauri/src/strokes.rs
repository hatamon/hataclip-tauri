//! 前面へ送るキー列。送り終わるとき、修飾キーは指の状態に戻る。
//! 指が押したままの修飾キーを一度離す列は `arms`。そのあいだ、登録済みの Ctrl 付きキーは前面へ渡さない。

use crate::keys::{TypeAtom, TypeKey, TypeStep};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mod {
    Ctrl,
    Shift,
    Alt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Held {
    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    fn get(self, modifier: Mod) -> bool {
        match modifier {
            Mod::Ctrl => self.ctrl,
            Mod::Shift => self.shift,
            Mod::Alt => self.alt,
        }
    }

    fn set(&mut self, modifier: Mod, down: bool) {
        match modifier {
            Mod::Ctrl => self.ctrl = down,
            Mod::Shift => self.shift = down,
            Mod::Alt => self.alt = down,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    Char(char),
    Insert,
    Home,
    Tab,
    Enter,
    Escape,
    Space,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stroke {
    Release(Mod),
    Press(Mod),
    Click(Click),
    Text(char),
    Wait(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script {
    pub strokes: Vec<Stroke>,
    /// 指が押したままの修飾キーを、列の途中で離す。
    pub arms: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Need {
    ctrl: bool,
    shift: bool,
    alt: bool,
}

impl Need {
    fn bare() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    fn get(self, modifier: Mod) -> bool {
        match modifier {
            Mod::Ctrl => self.ctrl,
            Mod::Shift => self.shift,
            Mod::Alt => self.alt,
        }
    }
}

enum Piece {
    Click(Click),
    Text(char),
}

pub fn plan_chord(ctrl: bool, shift: bool, key: Click, held: Held) -> Script {
    plan(
        &[(
            Need { ctrl, shift, alt: false },
            Piece::Click(key),
        )],
        held,
        &[],
    )
}

pub fn plan_text(text: &str, held: Held) -> Script {
    let pieces: Vec<(Need, Piece)> = text
        .chars()
        .map(|ch| {
            let piece = if ch == '\n' || ch == '\r' {
                Piece::Click(Click::Enter)
            } else if ch == '\t' {
                Piece::Click(Click::Tab)
            } else {
                Piece::Text(ch)
            };
            (Need::bare(), piece)
        })
        .collect();
    plan(&pieces, held, &[])
}

pub fn plan_atoms(atoms: &[TypeAtom], held: Held) -> Script {
    let mut pieces = Vec::new();
    let mut breaks = Vec::new();
    for (index, atom) in atoms.iter().enumerate() {
        if index > 0 {
            breaks.push(pieces.len());
        }
        match atom {
            TypeAtom::Text(text) => {
                for ch in text.chars() {
                    let piece = if ch == '\n' || ch == '\r' {
                        Piece::Click(Click::Enter)
                    } else if ch == '\t' {
                        Piece::Click(Click::Tab)
                    } else {
                        Piece::Text(ch)
                    };
                    pieces.push((Need::bare(), piece));
                }
            }
            TypeAtom::Key(step) => pieces.push((need_of(step), Piece::Click(click_of(step.key)))),
        }
    }
    plan(&pieces, held, &breaks)
}

fn need_of(step: &TypeStep) -> Need {
    Need {
        ctrl: step.ctrl,
        shift: step.shift,
        alt: step.alt,
    }
}

fn click_of(key: TypeKey) -> Click {
    match key {
        TypeKey::Char(ch) => Click::Char(ch),
        TypeKey::Tab => Click::Tab,
        TypeKey::Enter => Click::Enter,
        TypeKey::Escape => Click::Escape,
        TypeKey::Space => Click::Space,
        TypeKey::Backspace => Click::Backspace,
        TypeKey::Delete => Click::Delete,
        TypeKey::Insert => Click::Insert,
        TypeKey::Up => Click::Up,
        TypeKey::Down => Click::Down,
        TypeKey::Left => Click::Left,
        TypeKey::Right => Click::Right,
        TypeKey::Home => Click::Home,
        TypeKey::End => Click::End,
        TypeKey::PageUp => Click::PageUp,
        TypeKey::PageDown => Click::PageDown,
        TypeKey::F(n) => Click::F(n),
    }
}

/// `breaks` は、その位置の直前（修飾キーを指へ戻したあと）に 20ms 待つ。
fn plan(pieces: &[(Need, Piece)], held: Held, breaks: &[usize]) -> Script {
    let mut running = held;
    let mut strokes = Vec::new();
    let mut arms = false;
    for (index, (need, piece)) in pieces.iter().enumerate() {
        if breaks.contains(&index) {
            arms |= align(&mut running, need_from_held(held), held, &mut strokes);
            strokes.push(Stroke::Wait(20));
        }
        arms |= align(&mut running, *need, held, &mut strokes);
        strokes.push(match piece {
            Piece::Click(key) => Stroke::Click(*key),
            Piece::Text(ch) => Stroke::Text(*ch),
        });
    }
    arms |= align(&mut running, need_from_held(held), held, &mut strokes);
    debug_assert_eq!(running, held);
    Script { strokes, arms }
}

fn need_from_held(held: Held) -> Need {
    Need {
        ctrl: held.ctrl,
        shift: held.shift,
        alt: held.alt,
    }
}

const MODS: [Mod; 3] = [Mod::Ctrl, Mod::Shift, Mod::Alt];

fn align(running: &mut Held, need: Need, physical: Held, strokes: &mut Vec<Stroke>) -> bool {
    let mut arms = false;
    for modifier in MODS {
        if running.get(modifier) && !need.get(modifier) {
            strokes.push(Stroke::Release(modifier));
            running.set(modifier, false);
            if physical.get(modifier) {
                arms = true;
            }
        }
    }
    for modifier in MODS {
        if !running.get(modifier) && need.get(modifier) {
            strokes.push(Stroke::Press(modifier));
            running.set(modifier, true);
        }
    }
    arms
}

#[cfg(test)]
fn ends_like(held: Held, strokes: &[Stroke]) -> Held {
    let mut state = held;
    for stroke in strokes {
        match stroke {
            Stroke::Release(modifier) => state.set(*modifier, false),
            Stroke::Press(modifier) => state.set(*modifier, true),
            Stroke::Click(_) | Stroke::Text(_) | Stroke::Wait(_) => {}
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held_ctrl() -> Held {
        Held {
            ctrl: true,
            ..Held::none()
        }
    }

    #[test]
    fn shift_home_releases_a_held_ctrl_and_puts_it_back() {
        let script = plan_chord(false, true, Click::Home, held_ctrl());
        assert!(script.arms);
        assert_eq!(
            script.strokes,
            vec![
                Stroke::Release(Mod::Ctrl),
                Stroke::Press(Mod::Shift),
                Stroke::Click(Click::Home),
                Stroke::Release(Mod::Shift),
                Stroke::Press(Mod::Ctrl),
            ]
        );
        assert_eq!(ends_like(held_ctrl(), &script.strokes), held_ctrl());
    }

    #[test]
    fn shift_home_without_ctrl_only_presses_shift() {
        let script = plan_chord(false, true, Click::Home, Held::none());
        assert!(!script.arms);
        assert_eq!(
            script.strokes,
            vec![
                Stroke::Press(Mod::Shift),
                Stroke::Click(Click::Home),
                Stroke::Release(Mod::Shift),
            ]
        );
        assert_eq!(ends_like(Held::none(), &script.strokes), Held::none());
    }

    #[test]
    fn ctrl_v_keeps_a_held_ctrl() {
        let script = plan_chord(true, false, Click::Char('v'), held_ctrl());
        assert!(!script.arms);
        assert_eq!(script.strokes, vec![Stroke::Click(Click::Char('v'))]);
        assert_eq!(ends_like(held_ctrl(), &script.strokes), held_ctrl());
    }

    #[test]
    fn text_and_tab_while_ctrl_is_held_restore_ctrl() {
        let held = held_ctrl();
        let script = plan_text("ab", held);
        assert!(script.arms);
        assert_eq!(
            script.strokes,
            vec![
                Stroke::Release(Mod::Ctrl),
                Stroke::Text('a'),
                Stroke::Text('b'),
                Stroke::Press(Mod::Ctrl),
            ]
        );
        let tab = TypeStep {
            ctrl: false,
            shift: false,
            alt: false,
            key: TypeKey::Tab,
        };
        let atoms = plan_atoms(&[TypeAtom::Text("id".into()), TypeAtom::Key(tab)], held);
        assert!(atoms.arms);
        assert_eq!(
            atoms.strokes,
            vec![
                Stroke::Release(Mod::Ctrl),
                Stroke::Text('i'),
                Stroke::Text('d'),
                Stroke::Press(Mod::Ctrl),
                Stroke::Wait(20),
                Stroke::Release(Mod::Ctrl),
                Stroke::Click(Click::Tab),
                Stroke::Press(Mod::Ctrl),
            ]
        );
        assert_eq!(ends_like(held, &atoms.strokes), held);
    }

    #[test]
    fn ctrl_letter_while_ctrl_is_held_does_not_release() {
        let step = TypeStep {
            ctrl: true,
            shift: false,
            alt: false,
            key: TypeKey::Char('a'),
        };
        let script = plan_atoms(&[TypeAtom::Key(step)], held_ctrl());
        assert!(!script.arms);
        assert_eq!(script.strokes, vec![Stroke::Click(Click::Char('a'))]);
    }

    #[test]
    fn held_shift_is_lifted_for_plain_text_and_restored() {
        let held = Held {
            shift: true,
            ..Held::none()
        };
        let script = plan_text("a", held);
        assert!(script.arms);
        assert_eq!(
            script.strokes,
            vec![
                Stroke::Release(Mod::Shift),
                Stroke::Text('a'),
                Stroke::Press(Mod::Shift),
            ]
        );
        assert_eq!(ends_like(held, &script.strokes), held);
    }

    #[test]
    fn enigo_is_constructed_only_by_the_gate() {
        let gate = include_str!("platform/input.rs");
        assert_eq!(gate.matches("Enigo::new").count(), 1);
        let others = [
            include_str!("lib.rs"),
            include_str!("main.rs"),
            include_str!("bin/hataclip-gui.rs"),
            include_str!("actions.rs"),
            include_str!("chord.rs"),
            include_str!("cli.rs"),
            include_str!("cli_args.rs"),
            include_str!("clipboard.rs"),
            include_str!("crypt.rs"),
            include_str!("ctrl_gap.rs"),
            include_str!("editor.rs"),
            include_str!("eval.rs"),
            include_str!("expr.rs"),
            include_str!("help.rs"),
            include_str!("keys.rs"),
            include_str!("page.rs"),
            include_str!("pipe.rs"),
            include_str!("settings.rs"),
            include_str!("shell.rs"),
            include_str!("shortcuts.rs"),
            include_str!("store.rs"),
            include_str!("text.rs"),
            include_str!("tray.rs"),
            include_str!("platform/mod.rs"),
            include_str!("platform/windows.rs"),
            include_str!("platform/hook.rs"),
            include_str!("platform/unsupported.rs"),
        ];
        for src in others {
            assert!(!src.contains("Enigo::new"));
            assert!(!src.contains("SendInput"));
        }
    }
}
