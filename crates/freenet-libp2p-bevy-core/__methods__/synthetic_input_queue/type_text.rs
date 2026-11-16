use bevy::input::ButtonState;

use crate::testing;

pub fn type_text(queue: &mut testing::SyntheticInputQueue, text: &str) {
    for character in text.chars() {
        if testing::TypedKey::from(character).needs_shift {
            queue.pending.push_back(testing::SyntheticInput::Named(
                testing::NamedKey::ShiftLeft,
                ButtonState::Pressed,
            ));
        }
        queue.pending.push_back(testing::SyntheticInput::Key(
            character,
            ButtonState::Pressed,
        ));
        queue.pending.push_back(testing::SyntheticInput::Key(
            character,
            ButtonState::Released,
        ));
        if testing::TypedKey::from(character).needs_shift {
            queue.pending.push_back(testing::SyntheticInput::Named(
                testing::NamedKey::ShiftLeft,
                ButtonState::Released,
            ));
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
        super::type_text(&mut queue, "ab");
        assert_eq!(queue.pending.len(), 4);
    }

    #[test]
    fn test_uppercase_wraps_key_in_shift_press_and_release() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::type_text(&mut queue, "B");
        assert_eq!(
            queue.pending.iter().collect::<Vec<_>>(),
            vec![
                &testing::SyntheticInput::Named(testing::NamedKey::ShiftLeft, ButtonState::Pressed),
                &testing::SyntheticInput::Key('B', ButtonState::Pressed),
                &testing::SyntheticInput::Key('B', ButtonState::Released),
                &testing::SyntheticInput::Named(
                    testing::NamedKey::ShiftLeft,
                    ButtonState::Released
                ),
            ]
        );
    }
}
