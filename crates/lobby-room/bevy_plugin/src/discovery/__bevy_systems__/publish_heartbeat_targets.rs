use bevy::prelude::*;

use crate::discovery;

pub fn publish_heartbeat_targets(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    sender: Option<Res<discovery::HeartbeatTargetSender>>,
) {
    let Some(state) = state else {
        return;
    };
    let Some(sender) = sender else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let target = state.heartbeat_targets();
    sender.send_if_modified(|current| {
        let changed = *current != target;
        if changed {
            current.clone_from(&target);
        }
        changed
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::publish_heartbeat_targets;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    fn state_with_room() -> discovery::state_machine::DiscoveryState {
        let mut state = discovery::state_machine::DiscoveryState::from_ready_peer(
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")],
            },
            &discovery::Config::default(),
        );
        state.display_name = identity::Username::from("me");
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("room-a"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        state
    }

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::watch::channel(Vec::new());
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state_with_room());
        app.insert_resource(discovery::HeartbeatTargetSender(tx));
        app.add_systems(Update, publish_heartbeat_targets);
        app.update();
        assert_eq!(rx.borrow().len(), 1);
    }

    #[test]
    fn unchanged_state_sends_nothing_new() {
        let (tx, rx) = tokio::sync::watch::channel(Vec::new());
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state_with_room());
        app.insert_resource(discovery::HeartbeatTargetSender(tx));
        app.add_systems(Update, publish_heartbeat_targets);
        app.update();
        let first = rx.borrow().clone();
        app.update();
        let second = rx.borrow().clone();
        assert_eq!(first, second);
    }
}
