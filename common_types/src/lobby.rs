use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, DerefMut, From};
use serde::{Deserialize, Serialize};

use crate::LobbyRoom;
use crate::RoomMembership;
use crate::RoomName;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Deref, DerefMut, From)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[from(forward)]
pub struct Lobby(pub BTreeMap<RoomName, LobbyRoom>);

#[atomic_delegates]
impl Lobby {
    pub fn merge(&mut self, incoming: Self) {}
}

#[rustfmt::skip]
impl Lobby {
    pub fn drop_rooms_where_every_member_quit(&mut self) {
        self.retain(|_, room| {
            room.members
                .values()
                .any(|room_member| room_member.membership != RoomMembership::Departed)
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

    use super::Lobby;
    use crate::LobbyRoom;
    use crate::Peer;
    use crate::PeerId;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::RoomSize;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        let room_member = RoomMember {
            peer: Peer {
                id: PeerId::from("peer"),
                addrs: Vec::new(),
            },
            updated_at: UnixTime::from_secs(1),
            display_name: None,
            membership: RoomMembership::Occupant,
        };
        let incoming = Lobby::from([(
            RoomName::from("room-a"),
            LobbyRoom {
                room_size: RoomSize::DEFAULT,
                created_at: UnixTime::EPOCH,
                members: BTreeMap::from([(PeerId::from("peer"), room_member)]),
            },
        )]);
        let mut lobby = Lobby::default();
        lobby.merge(incoming);
        assert_eq!(lobby.len(), 1);
    }
}
