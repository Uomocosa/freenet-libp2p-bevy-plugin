use bevy::input::ButtonState;

use crate::testing;

pub fn follow(queue: &mut testing::SyntheticInputQueue, path: &testing::PointerPath) {
    let Some(&start) = path.points.first() else {
        return;
    };
    queue.move_cursor(start, path.speed);
    queue.pending.push_back(testing::SyntheticInput::Button(
        path.button,
        ButtonState::Pressed,
    ));
    let glides = path.points.windows(2).filter_map(|segment| match segment {
        [from, to] => Some(testing::SyntheticInput::Glide {
            from: *from,
            to: *to,
            speed: path.speed,
        }),
        _ => None,
    });
    queue.pending.extend(glides);
    queue.pending.push_back(testing::SyntheticInput::Button(
        path.button,
        ButtonState::Released,
    ));
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::mouse::MouseButton;
    use bevy::math::Vec2;

    use crate::testing;

    #[test]
    fn test_usage() {
        let path = testing::PointerPath {
            button: MouseButton::Left,
            points: vec![Vec2::ZERO, Vec2::new(50.0, 0.0), Vec2::new(50.0, 50.0)],
            speed: testing::PointerSpeed(100.0),
        };
        let mut queue = testing::SyntheticInputQueue::default();
        super::follow(&mut queue, &path);
        assert_eq!(queue.pending.len(), 5);
        assert_eq!(
            queue.pending.back(),
            Some(&testing::SyntheticInput::Button(
                MouseButton::Left,
                ButtonState::Released
            ))
        );
    }
}
