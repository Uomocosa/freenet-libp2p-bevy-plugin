use bevy::input::ButtonState;

use crate::testing;

pub fn type_text(queue: &mut testing::SyntheticInputQueue, text: &str) {
    queue.pending.extend(text.chars().flat_map(|character| {
        [
            testing::SyntheticInput::Key(character, ButtonState::Pressed),
            testing::SyntheticInput::Key(character, ButtonState::Released),
        ]
    }));
}

#[cfg(test)]
mod tests {
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        super::type_text(&mut queue, "ab");
        assert_eq!(queue.pending.len(), 4);
    }
}
