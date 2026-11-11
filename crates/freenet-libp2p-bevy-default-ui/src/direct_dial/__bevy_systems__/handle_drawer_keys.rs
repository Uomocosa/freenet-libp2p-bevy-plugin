use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::direct_dial;
use direct_dial::FriendsDrawerState;

pub fn handle_drawer_keys(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<FriendsDrawerState>) {
    let keys = keys.into_inner();
    if keys.just_pressed(KeyCode::Tab) {
        state.toggle();
    }
    if keys.just_pressed(KeyCode::Escape) && **state {
        state.close();
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::{KeyCode, KeyboardInput};
    use bevy::prelude::*;

    use super::handle_drawer_keys;
    use crate::direct_dial;
    use direct_dial::FriendsDrawerState;

    fn app_with(open: bool) -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
        app.insert_resource(FriendsDrawerState(open));
        app.add_systems(Update, handle_drawer_keys);
        app
    }

    fn press(app: &mut App, key: KeyCode) {
        app.world_mut().write_message(KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        app.world_mut().write_message(KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: ButtonState::Released,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(false);
        press(&mut app, KeyCode::Tab);
        assert!(**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn tab_toggles_the_drawer() {
        let mut app = app_with(false);
        press(&mut app, KeyCode::Tab);
        assert!(**app.world().resource::<FriendsDrawerState>());
        press(&mut app, KeyCode::Tab);
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn escape_closes_an_open_drawer() {
        let mut app = app_with(true);
        press(&mut app, KeyCode::Escape);
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn escape_leaves_a_closed_drawer_closed() {
        let mut app = app_with(false);
        press(&mut app, KeyCode::Escape);
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }
}
