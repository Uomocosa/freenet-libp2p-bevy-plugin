use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey};
use bevy::prelude::{App, Entity};

pub fn press_key(app: &mut App, key: KeyCode) {
    app.world_mut().write_message(KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
    app.world_mut().write_message(KeyboardInput {
        key_code: key,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonInput;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::KeyCode;
    use bevy::prelude::App;

    use super::press_key;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
        press_key(&mut app, KeyCode::Escape);
        assert!(
            app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_released(KeyCode::Escape)
        );
    }
}
