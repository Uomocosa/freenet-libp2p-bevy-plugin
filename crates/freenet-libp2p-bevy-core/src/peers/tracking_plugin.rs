use crate::p2p;
use crate::peers;
use bevy::prelude::App;
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TrackingPlugin;

impl bevy::prelude::Plugin for TrackingPlugin {
    fn build(&self, app: &mut App) {
        if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
            app.insert_resource(peers::PeerEventTap(channel.subscribe()));
        }
        app.add_systems(
            Update,
            (
                peers::bevy_systems::spawn_me_on_ready,
                peers::bevy_systems::track_peer_connections,
                peers::bevy_systems::despawn_orphan_peers,
                peers::bevy_systems::despawn_orphan_rooms,
            )
                .chain(),
        );
    }
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

#[cfg(test)]
mod build_peers_plugin_tests {
    use bevy::prelude::Plugin;
    use bevy::prelude::*;

    use crate::p2p;
    use crate::peers;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Libp2pEventChannel::default());
        Plugin::build(&peers::TrackingPlugin, &mut app);
        assert!(app.world().get_resource::<peers::PeerEventTap>().is_some());
    }
}
