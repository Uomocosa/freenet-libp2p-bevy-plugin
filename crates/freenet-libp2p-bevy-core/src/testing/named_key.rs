use bevy::input::keyboard::{Key, KeyCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedKey {
    Escape,
    Tab,
    Enter,
    Backspace,
    ShiftLeft,
}

impl NamedKey {
    #[must_use]
    pub const fn key_code(self) -> KeyCode {
        match self {
            Self::Escape => KeyCode::Escape,
            Self::Tab => KeyCode::Tab,
            Self::Enter => KeyCode::Enter,
            Self::Backspace => KeyCode::Backspace,
            Self::ShiftLeft => KeyCode::ShiftLeft,
        }
    }

    #[must_use]
    pub const fn logical_key(self) -> Key {
        match self {
            Self::Escape => Key::Escape,
            Self::Tab => Key::Tab,
            Self::Enter => Key::Enter,
            Self::Backspace => Key::Backspace,
            Self::ShiftLeft => Key::Shift,
        }
    }

    #[must_use]
    pub const fn key_text(self) -> Option<&'static str> {
        match self {
            Self::Escape => Some("\u{1b}"),
            Self::Tab => Some("\t"),
            Self::Enter => Some("\r"),
            Self::Backspace => Some("\u{8}"),
            Self::ShiftLeft => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::{Key, KeyCode};

    use super::NamedKey;

    #[test]
    fn test_usage() {
        assert_eq!(NamedKey::Escape.key_code(), KeyCode::Escape);
        assert_eq!(NamedKey::Backspace.logical_key(), Key::Backspace);
        assert_eq!(NamedKey::ShiftLeft.key_code(), KeyCode::ShiftLeft);
        assert_eq!(NamedKey::ShiftLeft.logical_key(), Key::Shift);
        assert_eq!(NamedKey::ShiftLeft.key_text(), None);
        assert_eq!(NamedKey::Tab.key_text(), Some("\t"));
        assert_eq!(NamedKey::Enter.key_text(), Some("\r"));
        assert_eq!(NamedKey::Backspace.key_text(), Some("\u{8}"));
        assert_eq!(NamedKey::Escape.key_text(), Some("\u{1b}"));
    }
}
