use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

#[derive(Clone, Copy)]
pub enum Action {
    Register,
    Show,
}

pub fn shortcut(action: Action) -> Shortcut {
    match action {
        Action::Register => {
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyY)
        }
        Action::Show => Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyL),
    }
}
