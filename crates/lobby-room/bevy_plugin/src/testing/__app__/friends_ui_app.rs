use bevy::prelude::*;
use freenet_libp2p_bevy_core::direct_dial;
use freenet_libp2p_bevy_core::time;

use crate::ui;

pub fn friends_ui_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<direct_dial::state_machine::Input>();
    app.add_message::<direct_dial::FriendAddressSubmitted>();
    app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
        &direct_dial::PeerBook::default(),
        time::UnixTime::from_secs(10),
    ));
    app.insert_resource(direct_dial::FriendAddressError(None));
    app.init_resource::<direct_dial::FriendsView>();
    app.init_resource::<direct_dial::OwnShareAddress>();
    app.add_plugins(ui::direct_dial::FriendsUiPlugin);
    app
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::ui;

    use super::friends_ui_app;

    #[test]
    fn test_usage() {
        let mut app = friends_ui_app();
        app.update();
        let mut drawers = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendsDrawer>>();
        assert_eq!(drawers.iter(app.world()).count(), 1);
    }
}
