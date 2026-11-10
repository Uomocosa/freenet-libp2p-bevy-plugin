use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy)]
pub struct FriendsUiPlugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for FriendsUiPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

#[atomic_delegates]
impl FriendsUiPlugin {
    pub fn build_plugin(self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::FriendsUiPlugin;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::time;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.insert_resource(core_direct_dial::state_machine::DialState::from_peer_book(
            &core_direct_dial::PeerBook::default(),
            time::UnixTime::from_secs(10),
        ));
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        app.add_plugins(FriendsUiPlugin);
        app.update();
        let mut tabs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendsTab>>();
        assert_eq!(tabs.iter(app.world()).count(), 1);
    }
}
