use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::direct_dial;

#[derive(Deref)]
pub struct FriendsPlugin(pub direct_dial::Config);

impl FriendsPlugin {
    #[must_use]
    pub const fn new(config: direct_dial::Config) -> Self {
        Self(config)
    }
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for FriendsPlugin {
    fn build(&self, app: &mut App) {
        self.build_friends_plugin(app);
    }
}

#[atomic_delegates]
impl FriendsPlugin {
    pub fn build_friends_plugin(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::FriendsPlugin;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(FriendsPlugin::new(direct_dial::Config::from(None)));
        app.update();
        assert!(
            app.world()
                .get_resource::<direct_dial::PeerBook>()
                .is_some()
        );
    }
}
