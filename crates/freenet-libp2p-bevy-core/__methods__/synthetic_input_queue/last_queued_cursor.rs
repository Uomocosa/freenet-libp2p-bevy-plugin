use bevy::math::Vec2;

use crate::testing;

pub fn last_queued_cursor(queue: &testing::SyntheticInputQueue) -> Option<Vec2> {
    queue
        .pending
        .iter()
        .rev()
        .find_map(|input| match input {
            testing::SyntheticInput::Glide { to, .. } => Some(*to),
            _ => None,
        })
        .or(queue.cursor)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::testing;

    #[test]
    fn test_usage() {
        let mut queue = testing::SyntheticInputQueue::default();
        assert_eq!(queue.last_queued_cursor(), None);
        queue.cursor = Some(Vec2::ONE);
        assert_eq!(queue.last_queued_cursor(), Some(Vec2::ONE));
    }
}
