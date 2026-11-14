use bevy::input::keyboard::{Key, KeyCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedKey {
    Escape,
    Tab,
    Enter,
    Backspace,
}

impl NamedKey {
    #[must_use]
    pub const fn key_code(self) -> KeyCode {
        match self {
            Self::Escape => KeyCode::Escape,
            Self::Tab => KeyCode::Tab,
            Self::Enter => KeyCode::Enter,
            Self::Backspace => KeyCode::Backspace,
        }
    }

    #[must_use]
    pub const fn logical_key(self) -> Key {
        match self {
            Self::Escape => Key::Escape,
            Self::Tab => Key::Tab,
            Self::Enter => Key::Enter,
            Self::Backspace => Key::Backspace,
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
    }
}
