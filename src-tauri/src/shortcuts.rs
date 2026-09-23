use crate::settings::Shortcuts;
use crate::{
    actions, complete_from_selection, paste_from_selection, register_from_clipboard, show_picker,
    AppState,
};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn apply(app: &AppHandle, shortcuts: &Shortcuts) -> Result<(), String> {
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
            let state = app.state::<AppState>();
            register_from_clipboard(app, &state);
        })
        .map_err(|error| format!("{} を登録できない: {error}", shortcuts.register))?;

    global
        .on_shortcut(show, |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            let state = app.state::<AppState>();
            show_picker(app, &state);
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
                let state = app.state::<AppState>();
                paste_from_selection(app, &state);
            });
        }
    }

    if !shortcuts.complete.is_empty()
        && shortcuts.complete != shortcuts.register
        && shortcuts.complete != shortcuts.show
        && shortcuts.complete != shortcuts.expand
    {
        if let Ok(complete) = actions::parse(&shortcuts.complete) {
            let _ = global.on_shortcut(complete, |app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let state = app.state::<AppState>();
                complete_from_selection(app, &state);
            });
        }
    }

    Ok(())
}

pub fn pause(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

pub fn resume(app: &AppHandle, shortcuts: &Shortcuts) -> Result<(), String> {
    apply(app, shortcuts)
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
            let state = app.state::<AppState>();
            crate::paste_ranked(index, app, &state);
        });
    }
}
