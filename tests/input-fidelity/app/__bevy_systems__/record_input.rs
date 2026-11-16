use bevy::prelude::*;
use bevy::window::WindowEvent;

use crate::input_record::{InputRecord, KeyLabel, MouseButtonKind, PressState};
use crate::input_recorder::InputRecorder;

pub fn record_input(
    mut window_events: MessageReader<WindowEvent>,
    mut recorder: ResMut<InputRecorder>,
) {
    recorder.frames_ran = recorder.frames_ran.saturating_add(1);
    let records = &mut recorder.records;
    records.extend(window_events.read().filter_map(|event| match event {
        WindowEvent::CursorMoved(event) => Some(InputRecord::CursorMoved {
            x: event.position.x,
            y: event.position.y,
        }),
        WindowEvent::CursorEntered(_) => Some(InputRecord::CursorEntered),
        WindowEvent::CursorLeft(_) => Some(InputRecord::CursorLeft),
        WindowEvent::MouseButtonInput(event) => Some(InputRecord::MouseButton {
            button: MouseButtonKind::from(event.button),
            state: PressState::from(event.state),
        }),
        WindowEvent::KeyboardInput(event) => Some(InputRecord::Keyboard {
            physical: KeyLabel::from_key_code(event.key_code),
            logical: KeyLabel::from_logical_key(&event.logical_key),
            state: PressState::from(event.state),
            text: event.text.as_ref().map(ToString::to_string),
            does_repeat: event.repeat,
        }),
        WindowEvent::WindowFocused(event) => Some(InputRecord::FocusChanged {
            is_focused: event.focused,
        }),
        _ => None,
    }));
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
    use bevy::input::mouse::{MouseButton, MouseButtonInput};
    use bevy::prelude::*;
    use bevy::window::{CursorEntered, CursorMoved, WindowEvent};

    use super::record_input;
    use crate::input_record::InputRecord;
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InputRecorder>()
            .add_message::<WindowEvent>()
            .add_systems(Update, record_input);
        let window = app.world_mut().spawn_empty().id();
        app.world_mut()
            .write_message(WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(120.0, 80.0),
                delta: None,
            }));
        app.world_mut()
            .write_message(WindowEvent::CursorEntered(CursorEntered { window }));
        app.world_mut()
            .write_message(WindowEvent::MouseButtonInput(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window,
            }));
        app.world_mut()
            .write_message(WindowEvent::KeyboardInput(KeyboardInput {
                key_code: KeyCode::KeyA,
                logical_key: Key::Character("a".into()),
                state: ButtonState::Pressed,
                text: Some("a".into()),
                repeat: false,
                window,
            }));
        app.update();
        let recorder = app.world().resource::<InputRecorder>();
        assert_eq!(recorder.frames_ran, 1);
        assert_eq!(recorder.records.len(), 4);
        assert_eq!(
            recorder.records.first(),
            Some(&InputRecord::CursorMoved { x: 120.0, y: 80.0 })
        );
        assert_eq!(recorder.records.get(1), Some(&InputRecord::CursorEntered));
        assert!(matches!(
            recorder.records.get(3),
            Some(InputRecord::Keyboard { .. })
        ));
    }
}
