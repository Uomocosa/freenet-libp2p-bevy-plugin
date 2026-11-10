use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TrackingPlugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for TrackingPlugin {
    fn build(&self, app: &mut App) {
        self.build_peers_plugin(app);
    }
}

#[atomic_delegates]
impl TrackingPlugin {
    pub fn build_peers_plugin(self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::TrackingPlugin;
    use crate::p2p;
    use crate::peers;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Libp2pEventChannel::default());
        app.add_plugins(TrackingPlugin);
        app.update();
        assert!(app.world().get_resource::<peers::PeerEventTap>().is_some());
    }
}
