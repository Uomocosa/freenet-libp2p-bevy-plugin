use bevy::prelude::*;

use crate::p2p;
use crate::peers;

pub fn build_peers_plugin(_plugin: &peers::TrackingPlugin, app: &mut App) {
    app.init_resource::<peers::PeerIndex>();
    app.init_resource::<peers::RoomIndex>();
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
        ),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_peers_plugin;
    use crate::p2p;
    use crate::peers;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Libp2pEventChannel::default());
        build_peers_plugin(&peers::TrackingPlugin, &mut app);
        assert!(app.world().get_resource::<peers::PeerIndex>().is_some());
        assert!(app.world().get_resource::<peers::RoomIndex>().is_some());
        assert!(app.world().get_resource::<peers::PeerEventTap>().is_some());
    }
}
