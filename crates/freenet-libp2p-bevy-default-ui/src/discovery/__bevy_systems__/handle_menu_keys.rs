use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::discovery;
use discovery::{EscMenuFriendsButton, EscMenuState, LeaveDialog};

pub fn handle_menu_keys(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut menu: ResMut<EscMenuState>,
    mut drawer: Option<ResMut<direct_dial::FriendsDrawerState>>,
    dialog: Option<Res<State<LeaveDialog>>>,
    next_dialog: Option<ResMut<NextState<LeaveDialog>>>,
    friends: Query<Entity, With<EscMenuFriendsButton>>,
    mut commands: Commands,
) {
    let Some(keys) = keys else {
        return;
    };
    let keys = keys.into_inner();
    if keys.just_pressed(KeyCode::Escape) {
        let is_drawer_open = drawer.as_deref().is_some_and(|drawer| **drawer);
        let is_dialog_open = dialog.is_some_and(|dialog| **dialog == LeaveDialog::Open);
        if is_drawer_open {
            if let Some(drawer) = drawer.as_deref_mut() {
                drawer.close();
            }
        } else if is_dialog_open {
            if let Some(mut next_dialog) = next_dialog {
                next_dialog.set(LeaveDialog::Closed);
            }
        } else {
            menu.toggle();
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        if **menu {
            for entity in &friends {
                commands.entity(entity).insert(Pressed);
            }
        } else if let Some(drawer) = drawer.as_deref_mut() {
            drawer.toggle();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::{KeyCode, KeyboardInput};
    use bevy::prelude::*;

    use super::super::open_drawer_from_menu;
    use super::handle_menu_keys;
    use crate::direct_dial;
    use crate::discovery;
    use discovery::{EscMenuFriendsButton, EscMenuState, LeaveDialog};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
        app.init_resource::<EscMenuState>();
        app.add_systems(Update, handle_menu_keys);
        app
    }

    fn app_with(drawer_open: bool, dialog: Option<LeaveDialog>) -> App {
        let mut app = app();
        app.insert_resource(direct_dial::FriendsDrawerState(drawer_open));
        if let Some(dialog) = dialog {
            app.insert_resource(State::new(dialog));
            app.init_resource::<NextState<LeaveDialog>>();
        }
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

    fn is_menu_open(app: &App) -> bool {
        **app.world().resource::<EscMenuState>()
    }

    fn is_drawer_open(app: &App) -> bool {
        **app.world().resource::<direct_dial::FriendsDrawerState>()
    }

    fn is_dialog_closing(app: &App) -> bool {
        matches!(
            app.world().resource::<NextState<LeaveDialog>>(),
            NextState::Pending(LeaveDialog::Closed)
        )
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        press(&mut app, KeyCode::Escape);
        assert!(is_menu_open(&app));
    }

    #[test]
    fn escape_toggles_the_menu() {
        let mut app = app_with(false, None);
        press(&mut app, KeyCode::Escape);
        assert!(is_menu_open(&app));
        press(&mut app, KeyCode::Escape);
        assert!(!is_menu_open(&app));
    }

    #[test]
    fn escape_closes_an_open_drawer_and_leaves_the_menu_closed() {
        let mut app = app_with(true, None);
        press(&mut app, KeyCode::Escape);
        assert!(!is_drawer_open(&app));
        assert!(!is_menu_open(&app));
        press(&mut app, KeyCode::Escape);
        assert!(is_menu_open(&app));
    }

    #[test]
    fn escape_closes_an_open_leave_prompt_and_leaves_the_menu_closed() {
        let mut app = app_with(false, Some(LeaveDialog::Open));
        press(&mut app, KeyCode::Escape);
        assert!(is_dialog_closing(&app));
        assert!(!is_menu_open(&app));
    }

    #[test]
    fn a_tab_press_with_the_menu_open_opens_the_drawer_like_a_click() {
        let mut app = app_with(false, None);
        **app.world_mut().resource_mut::<EscMenuState>() = true;
        let button = app.world_mut().spawn(EscMenuFriendsButton).id();
        app.add_systems(Update, open_drawer_from_menu);
        press(&mut app, KeyCode::Tab);
        assert!(!is_menu_open(&app));
        assert!(is_drawer_open(&app));
        assert!(
            app.world().get::<bevy::ui::Pressed>(button).is_some(),
            "the Friends button is pressed like a click"
        );
    }

    #[test]
    fn a_tab_press_without_the_menu_toggles_the_drawer() {
        let mut app = app_with(false, None);
        press(&mut app, KeyCode::Tab);
        assert!(is_drawer_open(&app));
    }

    #[test]
    fn the_menu_works_without_a_friends_drawer() {
        let mut app = app();
        press(&mut app, KeyCode::Escape);
        assert!(is_menu_open(&app));
        press(&mut app, KeyCode::Tab);
        assert!(is_menu_open(&app));
    }
}
