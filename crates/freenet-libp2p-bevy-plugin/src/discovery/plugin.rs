use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::discovery;
use crate::p2p;
use discovery::Config;

#[derive(Deref)]
pub struct Plugin(pub Config);

impl Plugin {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self(config)
    }
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

#[atomic_delegates]
impl Plugin {
    pub fn build_plugin(&self, app: &mut App) {}
    pub(crate) fn spawn_lobby_task(&self, app: &mut App) {}
    pub(crate) fn swarm_channels(app: &App) -> Option<p2p::SwarmChannels> {}
}
// no test_usage necessary
