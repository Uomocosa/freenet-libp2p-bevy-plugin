use std::collections::BTreeMap;
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Reflect;
use derive_more::{Deref, DerefMut, From};
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect, Deref, DerefMut, From,
)]
#[from(forward)]
pub struct Lobby(pub BTreeMap<identity::RoomName, discovery::LobbyRoom>);

#[atomic_delegates]

impl Lobby {
    pub fn merge(&mut self, incoming: Self) {}
    pub fn drop_dead(&mut self, now: discovery::UnixTime, ttl: Duration) {}
}

fn newest_room_member(room: &discovery::LobbyRoom) -> discovery::UnixTime {
    room.members
        .values()
        .map(|room_member| room_member.updated_at)
        .max()
        .unwrap_or(discovery::UnixTime::from_secs(0))
}

#[rustfmt::skip]
impl Lobby {
    pub fn drop_rooms_where_every_member_quit(&mut self) {
        self.retain(|_, room| {
            room.members
                .values()
                .any(|room_member| room_member.membership != discovery::RoomMembership::Quit)
        });
    }
    pub fn evict_oldest_rooms_until_lobby_fits_capacity(&mut self, capacity: usize) {
        while self.len() > capacity {
            let oldest = self
                .iter()
                .min_by_key(|(name, room)| (newest_room_member(room), name.as_str()))
                .map(|(name, _)| name.clone());
            let Some(oldest) = oldest else {
                break;
            };
            self.remove(&oldest);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let room_member = discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            updated_at: discovery::UnixTime::from_secs(1),
            display_name: None,
            membership: discovery::RoomMembership::InRoom,
        };
        let incoming = discovery::Lobby::from([(
            identity::RoomName::from("room-a"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::from([(identity::PeerId::from("peer"), room_member)]),
            },
        )]);
        let mut lobby = discovery::Lobby::default();
        lobby.merge(incoming);
        assert_eq!(lobby.len(), 1);
        lobby.drop_dead(
            discovery::UnixTime::from_secs(500),
            discovery::HolderHeartbeatIntervals::default().ttl,
        );
        assert!(lobby.is_empty());
    }
}

// needed helper: the most recent member update of one room
