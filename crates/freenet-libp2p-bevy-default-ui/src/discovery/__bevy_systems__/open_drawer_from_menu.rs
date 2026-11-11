use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::discovery;
use discovery::EscMenuFriendsButton;

pub fn open_drawer_from_menu(
    presses: Query<(), (Added<Pressed>, With<EscMenuFriendsButton>)>,
    mut menu: ResMut<discovery::EscMenuState>,
    mut drawer: Option<ResMut<direct_dial::FriendsDrawerState>>,
) {
    if presses.is_empty() {
        return;
    }
    menu.close();
    if let Some(drawer) = drawer.as_deref_mut()
        && !**drawer
    {
        drawer.toggle();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::open_drawer_from_menu;
    use crate::direct_dial;
    use crate::discovery;
    use discovery::EscMenuFriendsButton;

    fn app_with(drawer_open: bool) -> App {
        let mut app = App::new();
        app.init_resource::<discovery::EscMenuState>();
        app.insert_resource(direct_dial::FriendsDrawerState(drawer_open));
        app.add_systems(Update, open_drawer_from_menu);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(false);
        **app.world_mut().resource_mut::<discovery::EscMenuState>() = true;
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(!**app.world().resource::<discovery::EscMenuState>());
        assert!(**app.world().resource::<direct_dial::FriendsDrawerState>());
    }

    #[test]
    fn a_friends_press_opens_a_closed_drawer() {
        let mut app = app_with(false);
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(**app.world().resource::<direct_dial::FriendsDrawerState>());
    }

    #[test]
    fn a_friends_press_never_closes_an_open_drawer() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuFriendsButton, Pressed));
        app.update();
        assert!(**app.world().resource::<direct_dial::FriendsDrawerState>());
    }

    #[test]
    fn without_a_press_the_menu_stays_open() {
        let mut app = app_with(false);
        **app.world_mut().resource_mut::<discovery::EscMenuState>() = true;
        app.world_mut().spawn(EscMenuFriendsButton);
        app.update();
        assert!(**app.world().resource::<discovery::EscMenuState>());
    }
}
