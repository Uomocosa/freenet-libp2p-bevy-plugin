use std::ops::Sub;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKeyCode};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};

use crate::testing;

pub fn drain_synthetic_input(
    time: Res<Time>,
    mut queue: ResMut<testing::SyntheticInputQueue>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut cursor_moved: MessageWriter<CursorMoved>,
    mut window_events: MessageWriter<WindowEvent>,
    mut buttons: MessageWriter<MouseButtonInput>,
    mut keys: MessageWriter<KeyboardInput>,
) {
    let window = window.into_inner();
    let delta_secs = time.into_inner().delta_secs();
    let Some(input) = queue.pending.front().copied() else {
        return;
    };
    match input {
        testing::SyntheticInput::Glide { from, to, speed } => {
            let current = queue.cursor.unwrap_or(from);
            let next = current.move_towards(to, *speed * delta_secs);
            queue.cursor = Some(next);
            if next == to {
                queue.pending.pop_front();
            }
            let event = CursorMoved {
                window,
                position: next,
                delta: Some(next.sub(current)),
            };
            cursor_moved.write(event.clone());
            window_events.write(WindowEvent::CursorMoved(event));
        }
        testing::SyntheticInput::Button(button, state) => {
            queue.pending.pop_front();
            let event = MouseButtonInput {
                button,
                state,
                window,
            };
            buttons.write(event);
            window_events.write(WindowEvent::MouseButtonInput(event));
        }
        testing::SyntheticInput::Key(character) => {
            queue.pending.pop_front();
            let event = KeyboardInput {
                key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                logical_key: Key::Character(character.to_string().into()),
                state: ButtonState::Pressed,
                text: Some(character.to_string().into()),
                repeat: false,
                window,
            };
            keys.write(event.clone());
            window_events.write(WindowEvent::KeyboardInput(event));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::input::ButtonState;
    use bevy::input::keyboard::KeyboardInput;
    use bevy::input::mouse::MouseButtonInput;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};

    use super::drain_synthetic_input;
    use crate::testing;

    #[derive(Resource, Default)]
    struct Seen {
        log: Vec<Step>,
        reads: usize,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Step {
        Moved(Vec2),
        Button(ButtonState),
    }

    fn record(
        mut cursors: MessageReader<CursorMoved>,
        mut buttons: MessageReader<MouseButtonInput>,
        mut seen: ResMut<Seen>,
    ) {
        seen.reads = seen.reads.saturating_add(1);
        seen.log
            .extend(cursors.read().map(|event| Step::Moved(event.position)));
        seen.log
            .extend(buttons.read().map(|event| Step::Button(event.state)));
    }

    fn app_with_window() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .init_resource::<testing::SyntheticInputQueue>()
            .init_resource::<Seen>()
            .add_message::<CursorMoved>()
            .add_message::<WindowEvent>()
            .add_message::<MouseButtonInput>()
            .add_message::<KeyboardInput>()
            .add_systems(PreUpdate, (drain_synthetic_input, record).chain());
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_window();
        {
            let mut queue = app
                .world_mut()
                .resource_mut::<testing::SyntheticInputQueue>();
            queue.cursor = Some(Vec2::ZERO);
            queue.move_cursor(Vec2::new(100.0, 0.0), testing::PointerSpeed(100.0));
        }
        for _ in 0..15 {
            app.update();
        }
        assert!(
            app.world()
                .resource::<testing::SyntheticInputQueue>()
                .is_idle()
        );
        let positions: Vec<Vec2> = app
            .world()
            .resource::<Seen>()
            .log
            .iter()
            .filter_map(|step| match step {
                Step::Moved(position) => Some(*position),
                Step::Button(_) => None,
            })
            .collect();
        assert_eq!(positions.last(), Some(&Vec2::new(100.0, 0.0)));
        assert!(
            positions
                .windows(2)
                .all(|pair| matches!(pair, [before, after] if before.x < after.x))
        );
    }

    #[test]
    fn test_follow_presses_before_moves_and_releases_last() {
        let mut app = app_with_window();
        app.world_mut()
            .resource_mut::<testing::SyntheticInputQueue>()
            .follow(&testing::PointerPath {
                button: bevy::input::mouse::MouseButton::Left,
                points: vec![Vec2::ZERO, Vec2::new(50.0, 0.0)],
                speed: testing::PointerSpeed(100.0),
            });
        for _ in 0..20 {
            app.update();
        }
        let log = &app.world().resource::<Seen>().log;
        let pressed = log
            .iter()
            .position(|step| *step == Step::Button(ButtonState::Pressed));
        let released = log
            .iter()
            .position(|step| *step == Step::Button(ButtonState::Released));
        assert!(pressed.is_some_and(|index| index > 0));
        assert!(released.is_some_and(|index| Some(index) == log.len().checked_sub(1)));
    }
}
