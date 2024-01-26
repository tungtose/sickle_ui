use bevy::input::{keyboard::KeyCode, Input};

pub trait SymmetricKeysExt {
    fn symmetry_pressed(&self, input: KeyCode) -> bool;
    fn symmetry_just_pressed(&self, input: KeyCode) -> bool;
    fn symmetry_just_released(&self, input: KeyCode) -> bool;
}

impl SymmetricKeysExt for Input<KeyCode> {
    fn symmetry_pressed(&self, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::AltLeft | KeyCode::AltRight => {
                self.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
            }
            KeyCode::SuperLeft | KeyCode::SuperRight => {
                self.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
            }
            _ => self.pressed(keycode),
        }
    }

    fn symmetry_just_pressed(&self, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::AltLeft | KeyCode::AltRight => {
                self.any_just_pressed([KeyCode::AltLeft, KeyCode::AltRight])
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.any_just_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
            }
            KeyCode::SuperLeft | KeyCode::SuperRight => {
                self.any_just_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
            }
            _ => self.just_pressed(keycode),
        }
    }

    fn symmetry_just_released(&self, keycode: KeyCode) -> bool {
        match keycode {
            KeyCode::AltLeft | KeyCode::AltRight => {
                self.any_just_released([KeyCode::AltLeft, KeyCode::AltRight])
            }
            KeyCode::ControlLeft | KeyCode::ControlRight => {
                self.any_just_released([KeyCode::ControlLeft, KeyCode::ControlRight])
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                self.any_just_released([KeyCode::ShiftLeft, KeyCode::ShiftRight])
            }
            KeyCode::SuperLeft | KeyCode::SuperRight => {
                self.any_just_released([KeyCode::SuperLeft, KeyCode::SuperRight])
            }
            _ => self.just_released(keycode),
        }
    }
}

pub trait KeyCodeToStringExt {
    fn to_string(&self) -> String;
}

impl KeyCodeToStringExt for KeyCode {
    fn to_string(&self) -> String {
        let formatted = format!("{:?}", self);
        let formatted_str = formatted.as_str();

        let renamed = match self {
            KeyCode::Key1 => "1",
            KeyCode::Key2 => "2",
            KeyCode::Key3 => "3",
            KeyCode::Key4 => "4",
            KeyCode::Key5 => "5",
            KeyCode::Key6 => "6",
            KeyCode::Key7 => "7",
            KeyCode::Key8 => "8",
            KeyCode::Key9 => "9",
            KeyCode::Key0 => "0",
            KeyCode::Escape => "ESC",
            KeyCode::Insert => "Ins",
            KeyCode::Delete => "Del",
            KeyCode::Apostrophe => "'",
            KeyCode::Asterisk => "*",
            KeyCode::Plus => "+",
            KeyCode::At => "@",
            KeyCode::Backslash => "\\",
            KeyCode::Colon => ":",
            KeyCode::Comma => ",",
            KeyCode::NumpadDecimal => ".",
            KeyCode::NumpadDivide => "/",
            KeyCode::Equals => "=",
            KeyCode::Grave => "`",
            KeyCode::AltLeft => "Alt",
            KeyCode::BracketLeft => "[",
            KeyCode::ControlLeft => "Ctrl",
            KeyCode::ShiftLeft => "Shift",
            KeyCode::Minus => "-",
            KeyCode::NumpadMultiply => "*",
            KeyCode::NumpadComma => ",",
            KeyCode::NumpadEquals => "=",
            KeyCode::Period => ",",
            KeyCode::AltRight => "Alt",
            KeyCode::BracketRight => "]",
            KeyCode::ControlRight => "Ctrl",
            KeyCode::ShiftRight => "Shift",
            KeyCode::Semicolon => ";",
            KeyCode::Slash => "/",
            KeyCode::NumpadSubtract => "-",
            KeyCode::Underline => "_",
            _ => formatted_str,
        };

        renamed.to_string()
    }
}

pub trait ShortcutTextExt {
    fn shortcut_text(&self) -> String;
}

impl ShortcutTextExt for Vec<KeyCode> {
    fn shortcut_text(&self) -> String {
        self.iter()
            .map(|keycode| keycode.to_string())
            .collect::<Vec<String>>()
            .join("+")
    }
}
