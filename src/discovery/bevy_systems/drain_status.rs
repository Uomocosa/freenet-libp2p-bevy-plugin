use bevy::prelude::*;

use crate::discovery;

pub fn drain_status(feed: Res<discovery::StatusReceiver>, mut current: ResMut<discovery::Status>) {
    let feed = feed.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<discovery::Status> = None;
    while let Ok(status) = rx.try_recv() {
        latest = Some(status);
    }
    if let Some(status) = latest {
        *current = status;
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::sync::Mutex;

    use bevy::prelude::*;

    use super::drain_status;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Status>();
        app.insert_resource(discovery::StatusReceiver(Mutex::new(Some(rx))));
        app.add_systems(Update, drain_status);
        let status = discovery::Status {
            lobby: discovery::Lobby::default(),
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("r"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            connected: std::collections::BTreeSet::new(),
            freenet_peers: Vec::new(),
            is_lobby_attached: true,
            self_peer_id: None,
            lease_ttl: discovery::HolderHeartbeatIntervals::default().ttl,
        };
        tx.send(status.clone()).ok();
        app.update();
        assert_eq!(*app.world().resource::<discovery::Status>(), status);
    }
}
