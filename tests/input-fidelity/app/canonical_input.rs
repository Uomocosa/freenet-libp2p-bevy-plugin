use crate::input_record::{InputRecord, KeyLabel, MouseButtonKind, PressState};

#[derive(Debug, Clone, PartialEq)]
pub enum CanonicalInput {
    PointerMovedTo {
        x: f32,
        y: f32,
    },
    ButtonStateChanged {
        button: MouseButtonKind,
        state: PressState,
    },
    KeyStateChanged {
        physical: KeyLabel,
        logical: KeyLabel,
        state: PressState,
        text: Option<String>,
    },
}

impl CanonicalInput {
    #[must_use]
    pub fn from_records(records: &[InputRecord]) -> Vec<Self> {
        let mut canonical = Vec::new();
        let mut pending_pointer: Option<(f32, f32)> = None;
        for record in records {
            match record {
                InputRecord::CursorMoved { x, y } => {
                    pending_pointer = Some((x.round(), y.round()));
                }
                InputRecord::MouseButton { button, state } => {
                    if let Some((x, y)) = pending_pointer.take() {
                        canonical.push(Self::PointerMovedTo { x, y });
                    }
                    canonical.push(Self::ButtonStateChanged {
                        button: *button,
                        state: *state,
                    });
                }
                InputRecord::Keyboard {
                    physical,
                    logical,
                    state,
                    text,
                    does_repeat: _,
                } => {
                    if let Some((x, y)) = pending_pointer.take() {
                        canonical.push(Self::PointerMovedTo { x, y });
                    }
                    canonical.push(Self::KeyStateChanged {
                        physical: physical.clone(),
                        logical: logical.clone(),
                        state: *state,
                        text: text.clone(),
                    });
                }
                InputRecord::CursorEntered
                | InputRecord::CursorLeft
                | InputRecord::FocusChanged { is_focused: _ } => {}
            }
        }
        if let Some((x, y)) = pending_pointer.take() {
            canonical.push(Self::PointerMovedTo { x, y });
        }
        canonical
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::{Key, KeyCode, NativeKeyCode};

    use super::CanonicalInput;
    use crate::input_record::{InputRecord, KeyLabel, MouseButtonKind, PressState};

    #[test]
    fn test_usage() {
        let records = vec![
            InputRecord::FocusChanged { is_focused: true },
            InputRecord::CursorEntered,
            InputRecord::CursorMoved { x: 10.2, y: 20.7 },
            InputRecord::CursorMoved { x: 11.4, y: 21.1 },
            InputRecord::MouseButton {
                button: MouseButtonKind::Left,
                state: PressState::Pressed,
            },
            InputRecord::Keyboard {
                physical: KeyLabel::from_key_code(KeyCode::Unidentified(NativeKeyCode::Xkb(38))),
                logical: KeyLabel::from_logical_key(&Key::Character("a".into())),
                state: PressState::Pressed,
                text: Some("a".to_string()),
                does_repeat: false,
            },
            InputRecord::CursorLeft,
        ];
        let canonical = CanonicalInput::from_records(&records);
        assert_eq!(
            canonical,
            vec![
                CanonicalInput::PointerMovedTo { x: 11.0, y: 21.0 },
                CanonicalInput::ButtonStateChanged {
                    button: MouseButtonKind::Left,
                    state: PressState::Pressed,
                },
                CanonicalInput::KeyStateChanged {
                    physical: KeyLabel::from_key_code(KeyCode::Unidentified(NativeKeyCode::Xkb(
                        38
                    ))),
                    logical: KeyLabel::from_logical_key(&Key::Character("a".into())),
                    state: PressState::Pressed,
                    text: Some("a".to_string()),
                },
            ]
        );
        let collapsed = KeyLabel::from_key_code(KeyCode::Unidentified(NativeKeyCode::Unidentified));
        assert_ne!(
            KeyLabel::from_key_code(KeyCode::KeyA),
            KeyLabel::from_key_code(KeyCode::KeyB)
        );
        assert_ne!(
            KeyLabel::from_key_code(KeyCode::Unidentified(NativeKeyCode::Xkb(38))),
            collapsed
        );
    }
}
