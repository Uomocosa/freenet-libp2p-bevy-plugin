use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, DerefMut, From};
use serde::{Deserialize, Serialize};

use crate::Room;
use crate::RoomName;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Deref, DerefMut, From)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
#[from(forward)]
pub struct Lobby(pub BTreeMap<RoomName, Room>);

#[atomic_delegates]
impl Lobby {
    pub fn merge(&mut self, incoming: Self) {}
    pub fn drop_rooms_where_every_member_quit(&mut self) {}
    pub fn evict_oldest_rooms_until_lobby_fits_capacity(&mut self, capacity: usize) {}
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Lobby;
    use crate::Peer;
    use crate::PeerId;
    use crate::Room;
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
            Room {
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
