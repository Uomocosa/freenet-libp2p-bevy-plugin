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
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::RoomSize;
    use common_types::Peer;
    use common_types::PeerId;
    use common_types::UnixTime;
    use common_types::ZoneName;

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

    #[test]
    fn test_room_name_serializes_like_the_zone_name_it_wraps() {
        let room = RoomName::from("a");
        let zone = ZoneName::from("a");
        assert_eq!(
            bincode::serialize(&room).expect("room name encodes"),
            bincode::serialize(&zone).expect("zone name encodes")
        );
        assert_eq!(
            serde_json::to_string(&room).expect("room name encodes"),
            serde_json::to_string(&zone).expect("zone name encodes")
        );
        assert_eq!(RoomName::from(zone).as_str(), "a");
    }
}
