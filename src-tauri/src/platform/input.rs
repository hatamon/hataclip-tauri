use crate::chord::{self, ChordKey};
use crate::keys::TypeAtom;
use crate::strokes::{self, Click, Held, Mod, Script, Stroke};
use enigo::{
    Direction::{Click as PressClick, Press, Release},
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
    let key = match chord.key {
        ChordKey::Char(ch) => Click::Char(ch),
        ChordKey::Insert => Click::Insert,
        ChordKey::Home => Click::Home,
    };
    play(&strokes::plan_chord(chord.ctrl, chord.shift, key, held()))
}

/// 本文を 1 文字ずつ前面へ送る。2 秒を超えた文字は送らず、修飾キーは指へ戻す。
pub fn simulate_type(text: &str) -> bool {
    play(&strokes::plan_text(text, held()))
}

/// `{{type:}}` の断片を前面へ送る。
pub fn simulate_type_atoms(atoms: &[TypeAtom]) -> bool {
    play(&strokes::plan_atoms(atoms, held()))
}

fn enigo_settings() -> Settings {
    // 破棄時に Ctrl を上げると、指のキーアップが来なくなる。受け直しが残ったまま、4 と 7 が文字として入らなくなる。
    Settings {
        release_keys_when_dropped: false,
        ..Settings::default()
    }
}

fn held() -> Held {
    // 追跡だけ Ctrl が残っていると、貼り付けが V だけで、文字の v になる。
    if crate::ctrl_gap::sync_ctrl_with_finger() {
        release_control();
    }
    Held {
        ctrl: crate::ctrl_gap::physical(),
        shift: crate::ctrl_gap::physical_shift(),
        alt: crate::ctrl_gap::physical_alt(),
    }
}

fn with_enigo(body: impl FnOnce(&mut Enigo) -> bool) -> Result<bool, ()> {
    let mut enigo = Enigo::new(&enigo_settings()).map_err(|_| ())?;
    Ok(body(&mut enigo))
}

/// テスト終了時。WinAppDriver が離し損ねた Shift / Ctrl / Alt を上げる。
pub fn release_test_modifiers() {
    crate::ctrl_gap::begin_send();
    let _ = with_enigo(|enigo| {
        let keys = [
            Key::Control,
            Key::LControl,
            Key::RControl,
            Key::Shift,
            Key::LShift,
            Key::RShift,
            Key::Alt,
        ];
        for key in keys {
            let _ = enigo.key(key, Release);
        }
        true
    });
    crate::ctrl_gap::end_send();
}

/// 指が Ctrl を離したとき、列が押したままにした Ctrl を上げる。
pub(super) fn release_control() {
    let _ = with_enigo(|enigo| enigo.key(Key::Control, Release).is_ok());
}

/// キーを送る唯一の入口。
fn play(script: &Script) -> bool {
    // 列が指の Ctrl を離すときだけ受け直す。コピーだけの Ctrl+4 では離さない。
    if script.arms {
        crate::ctrl_gap::arm();
    }
    // テストモードでフックが注入キーを見るあいだ、この送信自体を指のキーと誤認しない。
    crate::ctrl_gap::begin_send();
    let result = match with_enigo(|enigo| {
        let started = Instant::now();
        let mut ok = true;
        for stroke in &script.strokes {
            let expired = started.elapsed() >= Duration::from_secs(2);
            let skip = expired && matches!(stroke, Stroke::Click(_) | Stroke::Text(_));
            if skip {
                ok = false;
                continue;
            }
            ok &= send(enigo, stroke);
        }
        ok
    }) {
        Ok(ok) => ok,
        Err(()) => {
            if script.arms {
                crate::ctrl_gap::disarm();
            }
            false
        }
    };
    crate::ctrl_gap::end_send();
    result
}

fn send(enigo: &mut Enigo, stroke: &Stroke) -> bool {
    match stroke {
        Stroke::Release(modifier) => enigo.key(modifier_key(*modifier), Release).is_ok(),
        Stroke::Press(modifier) => enigo.key(modifier_key(*modifier), Press).is_ok(),
        Stroke::Click(key) => enigo.key(click_key(*key), PressClick).is_ok(),
        Stroke::Text(ch) => enigo.text(&ch.to_string()).is_ok(),
        Stroke::Wait(ms) => {
            if *ms > 0 {
                std::thread::sleep(Duration::from_millis(*ms));
            }
            true
        }
    }
}

fn modifier_key(modifier: Mod) -> Key {
    match modifier {
        Mod::Ctrl => Key::Control,
        Mod::Shift => Key::Shift,
        Mod::Alt => Key::Alt,
    }
}

fn click_key(key: Click) -> Key {
    match key {
        Click::Char(ch) => Key::Unicode(ch),
        Click::Insert => Key::Insert,
        Click::Home => Key::Home,
        Click::Tab => Key::Tab,
        Click::Enter => Key::Return,
        Click::Escape => Key::Escape,
        Click::Space => Key::Space,
        Click::Backspace => Key::Backspace,
        Click::Delete => Key::Delete,
        Click::Up => Key::UpArrow,
        Click::Down => Key::DownArrow,
        Click::Left => Key::LeftArrow,
        Click::Right => Key::RightArrow,
        Click::End => Key::End,
        Click::PageUp => Key::PageUp,
        Click::PageDown => Key::PageDown,
        Click::F(n) => fn_key(n),
    }
}

fn fn_key(n: u8) -> Key {
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

#[cfg(test)]
mod tests {
    #[test]
    fn dropping_enigo_does_not_release_a_held_ctrl() {
        assert!(!super::enigo_settings().release_keys_when_dropped);
    }
}
