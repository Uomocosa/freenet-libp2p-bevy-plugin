use bevy::prelude::*;

use crate::net_id;
use crate::p2p;
use crate::roster;

pub fn poll_roster<T: p2p::Message>(
    mut roster: ResMut<roster::RoomRoster>,
    room: Res<roster::Room>,
    mut events: ResMut<p2p::Events<T>>,
    mut links: Local<std::collections::HashMap<net_id::PeerId, u32, std::hash::RandomState>>,
) {
    let mut rest = Vec::new();
    let room = room.into_inner();
    for event in events.take_all() {
        match event {
            p2p::Event::Net(p2p::NetEvent::PeerConnected(peer)) => {
                links
                    .entry(peer.clone())
                    .and_modify(|count| *count = count.saturating_add(1))
                    .or_insert(1);
                roster.add_entry((**room).clone(), peer_key(&peer), peer);
            }
            p2p::Event::Net(p2p::NetEvent::PeerDisconnected(peer)) => {
                let drained = links.get_mut(&peer).is_some_and(|count| {
                    *count = count.saturating_sub(1);
                    *count == 0
                });
                if drained {
                    links.remove(&peer);
                    roster.remove_entry(room, peer_key(&peer));
                }
            }
            p2p::Event::Net(p2p::NetEvent::Ready(me)) => {
                tracing::info!("ready peer_id={} addrs={:?}", me.id, me.addrs);
            }
            p2p::Event::Error(message) => {
                tracing::warn!("p2p error: {message}");
            }
            other => rest.push(other),
        }
    }
    events.extend(rest);
}

// needed helper: stable 32-byte roster key for a peer
fn peer_key(peer: &net_id::PeerId) -> [u8; 32] {
    *blake3::hash(peer.as_bytes()).as_bytes()
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use super::poll_roster;
    use crate::p2p;
    use crate::roster;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::RoomRoster::default());
        app.insert_resource(roster::Room(net_id::RoomName::from("alpha")));
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                net_id::PeerId::from("peer"),
            )));
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Message {
                from: net_id::PeerId::from("peer"),
                payload: Dummy(1),
            });
        app.add_systems(Update, poll_roster::<Dummy>);
        app.update();
        let alpha = app
            .world()
            .resource::<roster::RoomRoster>()
            .get(&net_id::RoomName::from("alpha"))
            .map_or(0, std::collections::BTreeMap::len);
        assert_eq!(alpha, 1);
        assert!(
            app.world()
                .resource::<roster::RoomRoster>()
                .get(&net_id::RoomName::from("default"))
                .is_none()
        );
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerDisconnected(
                net_id::PeerId::from("peer"),
            )));
        app.update();
        let members: usize = app
            .world()
            .resource::<roster::RoomRoster>()
            .values()
            .map(BTreeMap::len)
            .sum();
        assert_eq!(members, 0);
        assert_eq!(app.world().resource::<p2p::Events<Dummy>>().len(), 1);
    }

    #[test]
    fn sub_connection_drop_keeps_member() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(roster::RoomRoster::default());
        app.insert_resource(roster::Room(net_id::RoomName::from("alpha")));
        app.insert_resource(p2p::Events::<Dummy>::default());
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                net_id::PeerId::from("peer"),
            )));
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerConnected(
                net_id::PeerId::from("peer"),
            )));
        app.add_systems(Update, poll_roster::<Dummy>);
        app.update();
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerDisconnected(
                net_id::PeerId::from("peer"),
            )));
        app.update();
        let members: usize = app
            .world()
            .resource::<roster::RoomRoster>()
            .values()
            .map(BTreeMap::len)
            .sum();
        assert_eq!(members, 1);
        app.world_mut()
            .resource_mut::<p2p::Events<Dummy>>()
            .push(p2p::Event::Net(p2p::NetEvent::PeerDisconnected(
                net_id::PeerId::from("peer"),
            )));
        app.update();
        let members: usize = app
            .world()
            .resource::<roster::RoomRoster>()
            .values()
            .map(BTreeMap::len)
            .sum();
        assert_eq!(members, 0);
    }
}
