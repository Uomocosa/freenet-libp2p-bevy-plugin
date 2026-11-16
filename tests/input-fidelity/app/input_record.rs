use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode};
use bevy::input::mouse::MouseButton;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressState {
    Pressed,
    Released,
}

impl From<ButtonState> for PressState {
    fn from(state: ButtonState) -> Self {
        match state {
            ButtonState::Pressed => Self::Pressed,
            ButtonState::Released => Self::Released,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButtonKind {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<MouseButton> for MouseButtonKind {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::Back => Self::Back,
            MouseButton::Forward => Self::Forward,
            MouseButton::Other(other) => Self::Other(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyLabel(pub String);

impl KeyLabel {
    #[must_use]
    pub fn from_key_code(key_code: KeyCode) -> Self {
        Self(format!("{key_code:?}"))
    }

    #[must_use]
    pub fn from_logical_key(logical_key: &Key) -> Self {
        Self(format!("{logical_key:?}"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputRecord {
    CursorMoved {
        x: f32,
        y: f32,
    },
    CursorEntered,
    CursorLeft,
    MouseButton {
        button: MouseButtonKind,
        state: PressState,
    },
    Keyboard {
        physical: KeyLabel,
        logical: KeyLabel,
        state: PressState,
        text: Option<String>,
        does_repeat: bool,
    },
    FocusChanged {
        is_focused: bool,
    },
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyCode, NativeKey, NativeKeyCode};
    use bevy::input::mouse::MouseButton;

    use super::{InputRecord, KeyLabel, MouseButtonKind, PressState};

    #[test]
    fn test_usage() {
        assert_eq!(PressState::from(ButtonState::Pressed), PressState::Pressed);
        assert_eq!(
            MouseButtonKind::from(MouseButton::Left),
            MouseButtonKind::Left
        );
        assert_eq!(
            MouseButtonKind::from(MouseButton::Other(7)),
            MouseButtonKind::Other(7)
        );
        assert_eq!(
            KeyLabel::from_key_code(KeyCode::KeyA),
            KeyLabel("KeyA".to_string())
        );
        assert_eq!(
            KeyLabel::from_logical_key(&Key::Character("a".into())),
            KeyLabel("Character(\"a\")".to_string())
        );
        let record = InputRecord::Keyboard {
            physical: KeyLabel::from_key_code(KeyCode::Unidentified(NativeKeyCode::Unidentified)),
            logical: KeyLabel::from_logical_key(&Key::Unidentified(NativeKey::Unidentified)),
            state: PressState::Pressed,
            text: Some("a".to_string()),
            does_repeat: false,
        };
        let json = serde_json::to_value(&record).expect("record serialises");
        assert_eq!(json["Keyboard"]["text"], serde_json::Value::from("a"));
        let back: InputRecord = serde_json::from_value(json).expect("record round-trips");
        assert_eq!(back, record);
    }
}
