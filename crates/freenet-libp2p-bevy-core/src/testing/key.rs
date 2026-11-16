use bevy::input::keyboard::{self, KeyCode};
use serde::{Deserialize, Serialize};

use crate::testing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    Escape,
    Tab,
    Enter,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Ctrl,
    Shift,
    Alt,
    Super,
    CtrlRight,
    ShiftRight,
    AltRight,
    SuperRight,
    Char(char),
}

impl Key {
    #[must_use]
    pub fn key_code(self) -> KeyCode {
        match self {
            Self::Escape => KeyCode::Escape,
            Self::Tab => KeyCode::Tab,
            Self::Enter => KeyCode::Enter,
            Self::Backspace => KeyCode::Backspace,
            Self::Delete => KeyCode::Delete,
            Self::Insert => KeyCode::Insert,
            Self::Home => KeyCode::Home,
            Self::End => KeyCode::End,
            Self::PageUp => KeyCode::PageUp,
            Self::PageDown => KeyCode::PageDown,
            Self::ArrowUp => KeyCode::ArrowUp,
            Self::ArrowDown => KeyCode::ArrowDown,
            Self::ArrowLeft => KeyCode::ArrowLeft,
            Self::ArrowRight => KeyCode::ArrowRight,
            Self::F1 => KeyCode::F1,
            Self::F2 => KeyCode::F2,
            Self::F3 => KeyCode::F3,
            Self::F4 => KeyCode::F4,
            Self::F5 => KeyCode::F5,
            Self::F6 => KeyCode::F6,
            Self::F7 => KeyCode::F7,
            Self::F8 => KeyCode::F8,
            Self::F9 => KeyCode::F9,
            Self::F10 => KeyCode::F10,
            Self::F11 => KeyCode::F11,
            Self::F12 => KeyCode::F12,
            Self::Ctrl => KeyCode::ControlLeft,
            Self::Shift => KeyCode::ShiftLeft,
            Self::Alt => KeyCode::AltLeft,
            Self::Super => KeyCode::SuperLeft,
            Self::CtrlRight => KeyCode::ControlRight,
            Self::ShiftRight => KeyCode::ShiftRight,
            Self::AltRight => KeyCode::AltRight,
            Self::SuperRight => KeyCode::SuperRight,
            Self::Char(character) => testing::TypedKey::from(character).key_code,
        }
    }

    #[must_use]
    pub fn logical_key(self) -> keyboard::Key {
        match self {
            Self::Escape => keyboard::Key::Escape,
            Self::Tab => keyboard::Key::Tab,
            Self::Enter => keyboard::Key::Enter,
            Self::Backspace => keyboard::Key::Backspace,
            Self::Delete => keyboard::Key::Delete,
            Self::Insert => keyboard::Key::Insert,
            Self::Home => keyboard::Key::Home,
            Self::End => keyboard::Key::End,
            Self::PageUp => keyboard::Key::PageUp,
            Self::PageDown => keyboard::Key::PageDown,
            Self::ArrowUp => keyboard::Key::ArrowUp,
            Self::ArrowDown => keyboard::Key::ArrowDown,
            Self::ArrowLeft => keyboard::Key::ArrowLeft,
            Self::ArrowRight => keyboard::Key::ArrowRight,
            Self::F1 => keyboard::Key::F1,
            Self::F2 => keyboard::Key::F2,
            Self::F3 => keyboard::Key::F3,
            Self::F4 => keyboard::Key::F4,
            Self::F5 => keyboard::Key::F5,
            Self::F6 => keyboard::Key::F6,
            Self::F7 => keyboard::Key::F7,
            Self::F8 => keyboard::Key::F8,
            Self::F9 => keyboard::Key::F9,
            Self::F10 => keyboard::Key::F10,
            Self::F11 => keyboard::Key::F11,
            Self::F12 => keyboard::Key::F12,
            Self::Ctrl | Self::CtrlRight => keyboard::Key::Control,
            Self::Shift | Self::ShiftRight => keyboard::Key::Shift,
            Self::Alt | Self::AltRight => keyboard::Key::Alt,
            Self::Super | Self::SuperRight => keyboard::Key::Super,
            Self::Char(character) => testing::TypedKey::from(character).logical_key,
        }
    }

    #[must_use]
    pub fn text(self) -> Option<String> {
        match self {
            Self::Escape => Some("\u{1b}".to_string()),
            Self::Tab => Some("\t".to_string()),
            Self::Enter => Some("\r".to_string()),
            Self::Backspace => Some("\u{8}".to_string()),
            Self::Char(character) => Some(character.to_string()),
            Self::Delete
            | Self::Insert
            | Self::Home
            | Self::End
            | Self::PageUp
            | Self::PageDown
            | Self::ArrowUp
            | Self::ArrowDown
            | Self::ArrowLeft
            | Self::ArrowRight
            | Self::F1
            | Self::F2
            | Self::F3
            | Self::F4
            | Self::F5
            | Self::F6
            | Self::F7
            | Self::F8
            | Self::F9
            | Self::F10
            | Self::F11
            | Self::F12
            | Self::Ctrl
            | Self::Shift
            | Self::Alt
            | Self::Super
            | Self::CtrlRight
            | Self::ShiftRight
            | Self::AltRight
            | Self::SuperRight => None,
        }
    }

    #[must_use]
    pub fn needs_shift(self) -> bool {
        match self {
            Self::Char(character) => testing::TypedKey::from(character).needs_shift,
            Self::Escape
            | Self::Tab
            | Self::Enter
            | Self::Backspace
            | Self::Delete
            | Self::Insert
            | Self::Home
            | Self::End
            | Self::PageUp
            | Self::PageDown
            | Self::ArrowUp
            | Self::ArrowDown
            | Self::ArrowLeft
            | Self::ArrowRight
            | Self::F1
            | Self::F2
            | Self::F3
            | Self::F4
            | Self::F5
            | Self::F6
            | Self::F7
            | Self::F8
            | Self::F9
            | Self::F10
            | Self::F11
            | Self::F12
            | Self::Ctrl
            | Self::Shift
            | Self::Alt
            | Self::Super
            | Self::CtrlRight
            | Self::ShiftRight
            | Self::AltRight
            | Self::SuperRight => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::{Key as LogicalKey, KeyCode};

    use super::Key;

    #[test]
    fn test_usage() {
        assert_eq!(Key::Escape.key_code(), KeyCode::Escape);
        assert_eq!(Key::Backspace.logical_key(), LogicalKey::Backspace);
        assert_eq!(Key::Shift.key_code(), KeyCode::ShiftLeft);
        assert_eq!(Key::Shift.logical_key(), LogicalKey::Shift);
        assert_eq!(Key::Shift.text(), None);
        assert_eq!(Key::Tab.text(), Some("\t".to_string()));
        assert_eq!(Key::Enter.text(), Some("\r".to_string()));
        assert_eq!(Key::Backspace.text(), Some("\u{8}".to_string()));
        assert_eq!(Key::Escape.text(), Some("\u{1b}".to_string()));
        assert_eq!(Key::Ctrl.key_code(), KeyCode::ControlLeft);
        assert_eq!(Key::Char('a').key_code(), KeyCode::KeyA);
        assert_eq!(Key::Char(' ').logical_key(), LogicalKey::Space);
        assert!(Key::Char('B').needs_shift());
        assert!(!Key::Enter.needs_shift());
    }
}
