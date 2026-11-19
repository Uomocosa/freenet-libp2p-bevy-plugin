use std::ops::Sub;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};

use crate::testing;

pub fn drain_synthetic_input(
    time: Res<Time>,
    queue: ResMut<testing::SyntheticInputQueue>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut cursor_moved: MessageWriter<CursorMoved>,
    mut window_events: MessageWriter<WindowEvent>,
    mut buttons: MessageWriter<MouseButtonInput>,
    mut keys: MessageWriter<KeyboardInput>,
) {
    let window = window.into_inner();
    let delta = time.into_inner().delta();
    let delta_secs = delta.as_secs_f32();
    let queue = queue.into_inner();
    let Some(input) = queue.pending.front_mut() else {
        return;
    };
    let does_finish = match input {
        testing::SyntheticInput::Glide { from, to, speed } => {
            let current = queue.cursor.unwrap_or(*from);
            let next = current.move_towards(*to, **speed * delta_secs);
            queue.cursor = Some(next);
            let event = CursorMoved {
                window,
                position: next,
                delta: Some(next.sub(current)),
            };
            cursor_moved.write(event.clone());
            window_events.write(WindowEvent::CursorMoved(event));
            next == *to
        }
        testing::SyntheticInput::Button(button, state) => {
            let event = MouseButtonInput {
                button: *button,
                state: *state,
                window,
            };
            buttons.write(event);
            window_events.write(WindowEvent::MouseButtonInput(event));
            true
        }
        testing::SyntheticInput::Keys(events) => {
            for event in events.iter() {
                let pressed = KeyboardInput {
                    key_code: event.key.key_code(),
                    logical_key: event.key.logical_key(),
                    state: event.state,
                    text: if event.does_emit_text {
                        event.key.text().map(Into::into)
                    } else {
                        None
                    },
                    repeat: event.does_repeat,
                    window,
                };
                keys.write(pressed.clone());
                window_events.write(WindowEvent::KeyboardInput(pressed));
            }
            true
        }
        testing::SyntheticInput::Wait(remaining) => {
            let elapsed = u64::try_from(delta.as_nanos()).unwrap_or(u64::MAX);
            let remaining_nanos = (**remaining).saturating_mul(1_000_000);
            match remaining_nanos.checked_sub(elapsed) {
                Some(next) if next > 0 => {
                    let next_millis = next
                        .div_ceil(1_000_000)
                        .min((**remaining).saturating_sub(1));
                    if next_millis == 0 {
                        true
                    } else {
                        *remaining = testing::Millis::from(next_millis);
                        false
                    }
                }
                _ => true,
            }
        }
    };
    if does_finish {
        queue.pending.pop_front();
    }
}
