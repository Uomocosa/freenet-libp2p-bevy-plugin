use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorMoved, WindowEvent};

use crate::testing;

pub fn record_events(
    mut cursors: MessageReader<CursorMoved>,
    mut buttons: MessageReader<MouseButtonInput>,
    mut keys: MessageReader<KeyboardInput>,
    mut window_events: MessageReader<WindowEvent>,
    mut seen: ResMut<testing::Seen>,
) {
    seen.pointer.extend(
        cursors
            .read()
            .map(|event| testing::PointerStep::Moved(event.position)),
    );
    seen.pointer.extend(
        buttons
            .read()
            .map(|event| testing::PointerStep::Button(event.state)),
    );
    seen.keys
        .extend(keys.read().map(testing::RecordedKeyStep::from));
    seen.window_keys
        .extend(window_events.read().filter_map(|event| match event {
            WindowEvent::KeyboardInput(event) => Some(testing::RecordedKeyStep::from(event)),
            _ => None,
        }));
}
