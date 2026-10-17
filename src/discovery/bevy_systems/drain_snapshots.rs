use bevy::prelude::*;

use crate::discovery;

pub fn drain_snapshots(
    feed: Res<discovery::SnapshotReceiver>,
    mut current: ResMut<discovery::Snapshot>,
) {
    let feed = feed.into_inner();
    let Ok(mut guard) = feed.lock() else {
        return;
    };
    let Some(rx) = guard.as_mut() else {
        return;
    };
    let mut latest: Option<discovery::Snapshot> = None;
    while let Ok(snapshot) = rx.try_recv() {
        latest = Some(snapshot);
    }
    if let Some(snapshot) = latest {
        *current = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::sync::Mutex;

    use bevy::prelude::*;

    use super::drain_snapshots;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<discovery::Snapshot>();
        app.insert_resource(discovery::SnapshotReceiver(Mutex::new(Some(rx))));
        app.add_systems(Update, drain_snapshots);
        let snapshot = discovery::Snapshot {
            lobby: discovery::Lobby::default(),
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("r"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members: discovery::Members::new(),
            }),
            freenet_peers: Vec::new(),
        };
        tx.send(snapshot.clone()).ok();
        app.update();
        assert_eq!(*app.world().resource::<discovery::Snapshot>(), snapshot);
    }
}
