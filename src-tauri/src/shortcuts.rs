use crate::settings::Shortcuts;
use crate::{actions, register_from_clipboard, show_picker, AppState};
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

    Ok(())
}
