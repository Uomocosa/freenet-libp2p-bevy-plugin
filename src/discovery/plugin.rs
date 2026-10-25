use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::discovery;
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
    pub(crate) fn net_link(app: &App) -> Option<discovery::libp2p::NetLink> {}
}
// no test_usage necessary
