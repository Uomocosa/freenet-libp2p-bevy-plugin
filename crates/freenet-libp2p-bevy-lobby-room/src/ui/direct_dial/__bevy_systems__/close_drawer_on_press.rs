use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::direct_dial::{FriendsDrawerBackdrop, FriendsDrawerCloseButton, FriendsDrawerState};

type DrawerPresses = Or<(With<FriendsDrawerBackdrop>, With<FriendsDrawerCloseButton>)>;

pub fn close_drawer_on_press(
    presses: Query<(), (Added<Pressed>, DrawerPresses)>,
    mut state: ResMut<FriendsDrawerState>,
) {
    if presses.is_empty() {
        return;
    }
    state.close();
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::close_drawer_on_press;
    use ui::direct_dial::{FriendsDrawerBackdrop, FriendsDrawerCloseButton, FriendsDrawerState};

    fn app_with() -> App {
        let mut app = App::new();
        app.insert_resource(FriendsDrawerState(true));
        app.add_systems(Update, close_drawer_on_press);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with();
        app.world_mut().spawn((FriendsDrawerBackdrop, Pressed));
        app.update();
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn a_backdrop_press_closes_the_drawer() {
        let mut app = app_with();
        app.world_mut().spawn((FriendsDrawerBackdrop, Pressed));
        app.update();
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn a_close_button_press_closes_the_drawer() {
        let mut app = app_with();
        app.world_mut().spawn((FriendsDrawerCloseButton, Pressed));
        app.update();
        assert!(!**app.world().resource::<FriendsDrawerState>());
    }

    #[test]
    fn without_a_press_the_drawer_stays_open() {
        let mut app = app_with();
        app.world_mut().spawn(FriendsDrawerBackdrop);
        app.update();
        assert!(**app.world().resource::<FriendsDrawerState>());
    }
}
