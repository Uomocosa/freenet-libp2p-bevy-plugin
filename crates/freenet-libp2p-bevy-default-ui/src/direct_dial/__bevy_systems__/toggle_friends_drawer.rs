use bevy::prelude::*;

use crate::direct_dial;
use direct_dial::{FriendsDrawerState, ToggleFriendsDrawer};

pub fn toggle_friends_drawer(
    mut toggles: MessageReader<ToggleFriendsDrawer>,
    mut state: ResMut<FriendsDrawerState>,
) {
    for _ in toggles.read() {
        state.toggle();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::toggle_friends_drawer;
    use crate::direct_dial;
    use direct_dial::{FriendsDrawerState, ToggleFriendsDrawer};

    fn app_with(open: bool) -> App {
        let mut app = App::new();
        app.add_message::<ToggleFriendsDrawer>();
        app.insert_resource(FriendsDrawerState(open));
        app.add_systems(Update, toggle_friends_drawer);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(false);
        app.world_mut().write_message(ToggleFriendsDrawer);
        app.update();
        assert!(**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn each_message_toggles_once() {
        let mut app = app_with(false);
        app.world_mut().write_message(ToggleFriendsDrawer);
        app.world_mut().write_message(ToggleFriendsDrawer);
        app.update();
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn no_message_leaves_the_state() {
        let mut app = app_with(true);
        app.update();
        assert!(**app.world().resource::<FriendsDrawerState>());
    }
}
