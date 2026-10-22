use bevy::prelude::*;
use derive_more::Deref;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p};
use serde::{Deserialize, Serialize};

use crate::shared::status;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct PingPayload(pub identity::Username);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct IsRoomPingEnabled(pub bool);

/// The peers this one has received a room message from, for `wait-room-messages`.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct RoomMessageSenders {
    pub peers: std::collections::BTreeSet<identity::PeerId>,
}

pub fn publish_room_ping(
    time: Res<Time>,
    is_enabled: Res<IsRoomPingEnabled>,
    user: Res<status::LocalUser>,
    status: Res<discovery::Status>,
    mut commands: ResMut<p2p::Commands<PingPayload>>,
    mut accumulator: Local<f32>,
) {
    let time = time.into_inner();
    let is_enabled = is_enabled.into_inner();
    let user = user.into_inner();
    let status = status.into_inner();
    if !**is_enabled {
        return;
    }
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let is_connected_in_room = status.connected_members().next().is_some();
    if is_connected_in_room {
        commands.push(p2p::Command::PublishInJoinedRoom {
            payload: PingPayload(user.0.clone()),
        });
    }
}

pub fn log_room_messages(
    mut events: ResMut<p2p::Events<PingPayload>>,
    mut senders: ResMut<RoomMessageSenders>,
) {
    for event in events.take_all() {
        if let p2p::Event::RoomMessage {
            room,
            from,
            payload,
        } = event
        {
            tracing::info!(
                "lobby room_message room={} from={} user={}",
                room.as_str(),
                from.as_str(),
                payload.as_str()
            );
            senders.peers.insert(from);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity, p2p};

    use super::{IsRoomPingEnabled, PingPayload, log_room_messages, publish_room_ping};
    use crate::shared::status::LocalUser;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(IsRoomPingEnabled(true));
        app.insert_resource(LocalUser(identity::Username::from("alice")));
        let mut members = discovery::Members::new();
        members.insert(
            identity::PeerId::from("peer-b"),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from("peer-b"),
                    addrs: Vec::new(),
                },
                updated_at: discovery::UnixTime::from(0),
                display_name: Some(identity::Username::from("bob")),
                membership: discovery::RoomMembership::Occupant,
            },
        );
        app.insert_resource(discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members,
            }),
            connected: [identity::PeerId::from("peer-b")].into_iter().collect(),
            ..discovery::Status::default()
        });
        app.insert_resource(p2p::Commands::<PingPayload>::default());
        app.insert_resource(p2p::Events(vec![p2p::Event::RoomMessage {
            room: identity::RoomName::from("alpha"),
            from: identity::PeerId::from("peer-b"),
            payload: PingPayload(identity::Username::from("bob")),
        }]));
        app.init_resource::<super::RoomMessageSenders>();
        app.add_systems(Update, (publish_room_ping, log_room_messages));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.update();
        let commands = app.world().resource::<p2p::Commands<PingPayload>>();
        assert_eq!(commands.len(), 1);
        let events = app.world().resource::<p2p::Events<PingPayload>>();
        assert!(events.is_empty());
        assert_eq!(
            app.world()
                .resource::<super::RoomMessageSenders>()
                .peers
                .len(),
            1
        );
    }
}
