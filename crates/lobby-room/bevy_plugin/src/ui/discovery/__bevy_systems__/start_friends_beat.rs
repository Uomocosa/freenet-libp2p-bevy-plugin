use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::EscMenuFriendsButton;

pub fn start_friends_beat(
    presses: Query<(), (Added<Pressed>, With<EscMenuFriendsButton>)>,
    mut beat: ResMut<ui::discovery::FriendsPressBeat>,
) {
    if presses.is_empty() {
        return;
    }
    **beat = ui::discovery::MENU_FRIENDS_BEAT_SECS;
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::start_friends_beat;
    use ui::discovery::EscMenuFriendsButton;

    fn app_with(drawer_open: bool) -> App {
        let mut app = App::new();
        app.init_resource::<ui::discovery::EscMenuState>();
        app.init_resource::<ui::discovery::FriendsPressBeat>();
        app.insert_resource(ui::direct_dial::FriendsDrawerState(drawer_open));
        app.add_systems(Update, start_friends_beat);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(false);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::EscMenuState>() = true;
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(**app.world().resource::<ui::discovery::FriendsPressBeat>() > 0.0);
    }

    #[test]
    fn a_friends_press_keeps_the_menu_open_while_the_beat_runs() {
        let mut app = app_with(false);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::EscMenuState>() = true;
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(**app.world().resource::<ui::discovery::EscMenuState>());
        assert!(
            !**app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>()
        );
    }

    #[test]
    fn a_friends_press_does_not_move_the_drawer_itself() {
        let mut app = app_with(false);
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(
            !(**app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>())
        );
    }

    #[test]
    fn a_friends_press_never_touches_an_open_drawer() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(
            **app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>()
        );
    }

    #[test]
    fn without_a_press_the_beat_stays_idle() {
        let mut app = app_with(false);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::EscMenuState>() = true;
        app.world_mut().spawn(EscMenuFriendsButton);
        app.update();
        assert!((**app.world().resource::<ui::discovery::FriendsPressBeat>()).abs() < f32::EPSILON);
        assert!(**app.world().resource::<ui::discovery::EscMenuState>());
    }
}
