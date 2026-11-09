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
    pub fn build_plugin(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::FriendsUiPlugin;
    use crate::direct_dial;
    use crate::time;
    use crate::ui;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Input>();
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            &direct_dial::PeerBook::default(),
            time::UnixTime::from_secs(10),
        ));
        app.insert_resource(direct_dial::FriendAddressError(None));
        app.add_plugins(FriendsUiPlugin);
        app.update();
        let mut tabs = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendsTab>>();
        assert_eq!(tabs.iter(app.world()).count(), 1);
    }
}
