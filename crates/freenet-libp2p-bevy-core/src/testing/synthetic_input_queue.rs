use std::collections::VecDeque;

use atomic_delegate_macros::atomic_delegates;
use bevy::input::ButtonState;
use bevy::input::mouse::MouseButton;
use bevy::math::Vec2;
use bevy::prelude::Resource;

use crate::testing;

#[derive(Resource, Debug, Default)]
pub struct SyntheticInputQueue {
    pub cursor: Option<Vec2>,
    pub pending: VecDeque<testing::SyntheticInput>,
}

impl SyntheticInputQueue {
    pub fn move_cursor(&mut self, target: Vec2, speed: testing::PointerSpeed) {
        let from = self.last_queued_cursor().unwrap_or(target);
        self.pending.push_back(testing::SyntheticInput::Glide {
            from,
            to: target,
            speed,
        });
    }

    pub fn click_at(&mut self, target: Vec2, speed: testing::PointerSpeed) {
        self.move_cursor(target, speed);
        self.pending.push_back(testing::SyntheticInput::Button(
            MouseButton::Left,
            ButtonState::Pressed,
        ));
        self.pending.push_back(testing::SyntheticInput::Button(
            MouseButton::Left,
            ButtonState::Released,
        ));
    }
}

#[rustfmt::skip]
impl SyntheticInputQueue {
    #[must_use]
    pub fn is_idle(&self) -> bool { self.pending.is_empty() }
}

#[atomic_delegates]
impl SyntheticInputQueue {
    pub fn type_text(&mut self, text: &str) {}
    pub fn follow(&mut self, path: &testing::PointerPath) {}
}

#[atomic_delegates]
impl SyntheticInputQueue {
    pub(crate) fn last_queued_cursor(&self) -> Option<Vec2> {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        queue.move_cursor(Vec2::new(5.0, 0.0), testing::PointerSpeed(10.0));
        assert!(!queue.is_idle());
        assert_eq!(queue.last_queued_cursor(), Some(Vec2::new(5.0, 0.0)));
    }
}
