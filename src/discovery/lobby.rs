use std::collections::BTreeMap;

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
    pub fn drop_members_whose_lease_lapsed(
        &mut self,
        observed: &BTreeMap<
            (identity::RoomName, identity::PeerId),
            discovery::state_machine::ObservedLobbyHeartbeat,
        >,
        now: discovery::UnixTime,
        limits: discovery::LobbyLeaseLimits,
    ) {
    }
}

#[rustfmt::skip]
impl Lobby {
    pub fn drop_rooms_where_every_member_quit(&mut self) {
        self.retain(|_, room| {
            room.members
                .values()
                .any(|room_member| room_member.membership != discovery::RoomMembership::Departed)
        });
    }
    pub fn evict_oldest_rooms_until_lobby_fits_capacity(&mut self, capacity: usize) {
        while self.len() > capacity {
            let oldest = self
                .iter()
                .min_by_key(|(name, room)| (room.newest_room_member(), name.as_str()))
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
            updated_at: discovery::UnixTime::from(1),
            display_name: None,
            membership: discovery::RoomMembership::Occupant,
        };
        let incoming = discovery::Lobby::from([(
            identity::RoomName::from("room-a"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(0),
                members: BTreeMap::from([(identity::PeerId::from("peer"), room_member)]),
            },
        )]);
        let mut lobby = discovery::Lobby::default();
        lobby.merge(incoming);
        assert_eq!(lobby.len(), 1);
        let lease = discovery::HolderHeartbeatIntervals::default();
        lobby.drop_members_whose_lease_lapsed(
            &BTreeMap::new(),
            discovery::UnixTime::from(1000),
            discovery::LobbyLeaseLimits {
                lease_ttl: lease.ttl,
                remote_clock_skew_margin: lease.remote_clock_skew_margin,
            },
        );
        assert!(lobby.is_empty());
    }
}
