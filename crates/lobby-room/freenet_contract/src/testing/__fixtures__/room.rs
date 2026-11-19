use std::collections::BTreeMap;

use common_types::Peer;
use common_types::PeerId;
use common_types::UnixTime;

use crate::Room;
use crate::RoomMember;
use crate::RoomMembership;
use crate::RoomSize;

#[must_use]
pub fn room(peer: &str, updated_at: UnixTime, created_at: UnixTime) -> Room {
    let id = PeerId::from(peer);
    let mut members = BTreeMap::new();
    members.insert(
        id.clone(),
        RoomMember {
            peer: Peer {
                id,
                addrs: Vec::new(),
            },
            updated_at,
            display_name: None,
            membership: RoomMembership::Occupant,
        },
    );
    Room {
        room_size: RoomSize::DEFAULT,
        created_at,
        members,
    }
}

#[cfg(test)]
mod tests {
    use super::room;
    use crate::RoomMembership;
    use common_types::PeerId;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let room = room("peer", UnixTime::from_secs(5), UnixTime::from_secs(10));
        assert_eq!(room.members.len(), 1);
        assert_eq!(room.created_at, UnixTime::from_secs(10));
        let member = room
            .members
            .get(&PeerId::from("peer"))
            .expect("the member is present");
        assert_eq!(member.updated_at, UnixTime::from_secs(5));
        assert_eq!(member.membership, RoomMembership::Occupant);
    }
}
