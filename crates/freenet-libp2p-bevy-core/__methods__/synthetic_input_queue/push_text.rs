use bevy::input::ButtonState;

use crate::testing;

pub fn push_text(queue: &mut testing::SyntheticInputQueue, text: &str) {
    for character in text.chars() {
        let key = testing::Key::Char(character);
        if key.needs_shift() {
            queue
                .pending
                .push_back(testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Shift,
                    state: ButtonState::Pressed,
                    does_repeat: false,
                    does_emit_text: false,
                }]));
        }
        queue
            .pending
            .push_back(testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                key,
                state: ButtonState::Pressed,
                does_repeat: false,
                does_emit_text: true,
            }]));
        queue
            .pending
            .push_back(testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                key,
                state: ButtonState::Released,
                does_repeat: false,
                does_emit_text: true,
            }]));
        if key.needs_shift() {
            queue
                .pending
                .push_back(testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Shift,
                    state: ButtonState::Released,
                    does_repeat: false,
                    does_emit_text: false,
                }]));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_text(&mut queue, "ab");
        assert_eq!(queue.pending.len(), 4);
    }

    #[test]
    fn test_uppercase_wraps_key_in_shift_press_and_release() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::push_text(&mut queue, "B");
        assert_eq!(
            queue.pending.iter().collect::<Vec<_>>(),
            vec![
                &testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Shift,
                    state: ButtonState::Pressed,
                    does_repeat: false,
                    does_emit_text: false,
                }]),
                &testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Char('B'),
                    state: ButtonState::Pressed,
                    does_repeat: false,
                    does_emit_text: true,
                }]),
                &testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Char('B'),
                    state: ButtonState::Released,
                    does_repeat: false,
                    does_emit_text: true,
                }]),
                &testing::SyntheticInput::Keys(vec![testing::KeyEvent {
                    key: testing::Key::Shift,
                    state: ButtonState::Released,
                    does_repeat: false,
                    does_emit_text: false,
                }]),
            ]
        );
    }
}
