use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::{CursorEntered, CursorLeft, CursorMoved, WindowFocused};

use crate::input_record::{InputRecord, KeyLabel, MouseButtonKind, PressState};
use crate::input_recorder::InputRecorder;

pub fn record_input(
    mut cursors: MessageReader<CursorMoved>,
    mut entered: MessageReader<CursorEntered>,
    mut left: MessageReader<CursorLeft>,
    mut buttons: MessageReader<MouseButtonInput>,
    mut keys: MessageReader<KeyboardInput>,
    mut focused: MessageReader<WindowFocused>,
    mut recorder: ResMut<InputRecorder>,
) {
    recorder.frames_ran = recorder.frames_ran.saturating_add(1);
    let records = &mut recorder.records;
    records.extend(cursors.read().map(|event| InputRecord::CursorMoved {
        x: event.position.x,
        y: event.position.y,
    }));
    records.extend(entered.read().map(|_| InputRecord::CursorEntered));
    records.extend(left.read().map(|_| InputRecord::CursorLeft));
    records.extend(buttons.read().map(|event| InputRecord::MouseButton {
        button: MouseButtonKind::from(event.button),
        state: PressState::from(event.state),
    }));
    records.extend(keys.read().map(|event| InputRecord::Keyboard {
        physical: KeyLabel::from_key_code(event.key_code),
        logical: KeyLabel::from_logical_key(&event.logical_key),
        state: PressState::from(event.state),
        text: event.text.as_ref().map(ToString::to_string),
        does_repeat: event.repeat,
    }));
    records.extend(focused.read().map(|event| InputRecord::FocusChanged {
        is_focused: event.focused,
    }));
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
    use bevy::input::mouse::{MouseButton, MouseButtonInput};
    use bevy::prelude::*;
    use bevy::window::{CursorEntered, CursorLeft, CursorMoved, WindowFocused};

    use super::record_input;
    use crate::input_record::InputRecord;
    use crate::input_recorder::InputRecorder;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InputRecorder>()
            .add_message::<CursorMoved>()
            .add_message::<CursorEntered>()
            .add_message::<CursorLeft>()
            .add_message::<MouseButtonInput>()
            .add_message::<KeyboardInput>()
            .add_message::<WindowFocused>()
            .add_systems(Update, record_input);
        let window = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(CursorMoved {
            window,
            position: Vec2::new(120.0, 80.0),
            delta: None,
        });
        app.world_mut().write_message(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Pressed,
            window,
        });
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::KeyA,
            logical_key: Key::Character("a".into()),
            state: ButtonState::Pressed,
            text: Some("a".into()),
            repeat: false,
            window,
        });
        app.update();
        let recorder = app.world().resource::<InputRecorder>();
        assert_eq!(recorder.frames_ran, 1);
        assert_eq!(recorder.records.len(), 3);
        assert_eq!(
            recorder.records.first(),
            Some(&InputRecord::CursorMoved { x: 120.0, y: 80.0 })
        );
        assert!(matches!(
            recorder.records.get(2),
            Some(InputRecord::Keyboard { .. })
        ));
    }
}
