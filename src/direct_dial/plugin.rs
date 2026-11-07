use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::direct_dial;

#[derive(Deref)]
pub struct Plugin(pub direct_dial::Config);

impl Plugin {
    #[must_use]
    pub const fn new(config: direct_dial::Config) -> Self {
        Self(config)
    }
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        self.build_friends_plugin(app);
    }
}

#[atomic_delegates]
impl Plugin {
    pub fn build_friends_plugin(&self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::Plugin;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(Plugin::new(direct_dial::Config::from(None)));
        app.update();
        assert!(
            app.world()
                .get_resource::<direct_dial::PeerBook>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<direct_dial::ConnectedPeers>()
                .is_some()
        );
    }
}
