use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedKeyStep {
    pub key_code: KeyCode,
    pub logical_key: Key,
    pub state: ButtonState,
    pub text: Option<String>,
}

impl From<&KeyboardInput> for RecordedKeyStep {
    fn from(event: &KeyboardInput) -> Self {
        Self {
            key_code: event.key_code,
            logical_key: event.logical_key.clone(),
            state: event.state,
            text: event.text.as_ref().map(ToString::to_string),
        }
    }
}
