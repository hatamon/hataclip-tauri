use crate::actions::Action;
use crate::{register_from_clipboard, show_picker, AppState};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn register(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let register_shortcut = crate::actions::shortcut(Action::Register);
    let show_shortcut = crate::actions::shortcut(Action::Show);
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(register_shortcut, move |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            let state = app.state::<AppState>();
            register_from_clipboard(app, &state);
        })?;
    app.global_shortcut()
        .on_shortcut(show_shortcut, move |_app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            let state = handle.state::<AppState>();
            show_picker(&handle, &state);
        })?;
    Ok(())
}
