use crate::ctrl_gap::{self, Action, Chord};
use crate::settings::Shortcuts;
use crate::{
    actions, paste_from_selection, register_from_clipboard, show_picker, AppState,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn with_shortcut(run: impl FnOnce()) {
    static GATE: Mutex<()> = Mutex::new(());
    let _gate = GATE.lock().unwrap_or_else(|err| err.into_inner());
    run();
}

pub fn apply(app: &AppHandle, shortcuts: &Shortcuts) -> Result<(), String> {
    #[cfg(windows)]
    crate::platform::install_ctrl_gap_hook();
    let register = actions::parse(&shortcuts.register)?;
    let show = actions::parse(&shortcuts.show)?;
    if register == show {
        return Err("2 つに同じキーは割り当てられない".to_string());
    }

    let global = app.global_shortcut();
    let _ = global.unregister_all();

    global
        .on_shortcut(register, |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            with_shortcut(|| {
                let state = app.state::<AppState>();
                register_from_clipboard(app, &state);
            });
        })
        .map_err(|error| format!("{} を登録できない: {error}", shortcuts.register))?;

    global
        .on_shortcut(show, |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            with_shortcut(|| {
                let state = app.state::<AppState>();
                show_picker(app, &state);
            });
        })
        .map_err(|error| format!("{} を登録できない: {error}", shortcuts.show))?;

    if shortcuts.quick_paste {
        register_quick_paste(app);
    }

    if !shortcuts.expand.is_empty()
        && shortcuts.expand != shortcuts.register
        && shortcuts.expand != shortcuts.show
    {
        if let Ok(expand) = actions::parse(&shortcuts.expand) {
            let _ = global.on_shortcut(expand, |app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                with_shortcut(|| {
                    let state = app.state::<AppState>();
                    paste_from_selection(app, &state);
                });
            });
        }
    }

    publish_gap(app, shortcuts);
    Ok(())
}

pub fn pause(app: &AppHandle) {
    ctrl_gap::clear();
    let _ = app.global_shortcut().unregister_all();
}

pub fn resume(app: &AppHandle, shortcuts: &Shortcuts) -> Result<(), String> {
    apply(app, shortcuts)
}

fn publish_gap(app: &AppHandle, shortcuts: &Shortcuts) {
    let app = app.clone();
    let handler = Arc::new(move |action: Action| {
        with_shortcut(|| dispatch(&app, action));
    });
    ctrl_gap::publish(gap_chords(shortcuts), handler);
}

fn dispatch(app: &AppHandle, action: Action) {
    let state = app.state::<AppState>();
    match action {
        Action::Register => register_from_clipboard(app, &state),
        Action::Show => show_picker(app, &state),
        Action::Expand => paste_from_selection(app, &state),
        Action::Ranked(index) => crate::paste_ranked(index, app, &state),
    }
}

fn gap_chords(shortcuts: &Shortcuts) -> Vec<Chord> {
    let mut chords = Vec::new();
    push_chord(&mut chords, &shortcuts.register, Action::Register);
    push_chord(&mut chords, &shortcuts.show, Action::Show);
    if !shortcuts.expand.is_empty() {
        push_chord(&mut chords, &shortcuts.expand, Action::Expand);
    }
    if shortcuts.quick_paste {
        for digit in 1..=9 {
            push_chord(
                &mut chords,
                &format!("Control+Shift+Digit{digit}"),
                Action::Ranked(digit - 1),
            );
        }
    }
    chords
}

fn push_chord(chords: &mut Vec<Chord>, spec: &str, action: Action) {
    let Some((shift, alt, vk)) = ctrl_gap::parse_ctrl_shortcut(spec) else {
        return;
    };
    if chords
        .iter()
        .any(|chord| chord.vk == vk && chord.shift == shift && chord.alt == alt)
    {
        return;
    }
    chords.push(Chord {
        shift,
        alt,
        vk,
        action,
    });
}

fn register_quick_paste(app: &AppHandle) {
    let global = app.global_shortcut();
    for digit in 1..=9 {
        let spec = format!("Control+Shift+Digit{digit}");
        let Ok(shortcut) = actions::parse(&spec) else {
            continue;
        };
        let index = digit - 1;
        let _ = global.on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            with_shortcut(|| {
                let state = app.state::<AppState>();
                crate::paste_ranked(index, app, &state);
            });
        });
    }
}
