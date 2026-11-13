use crate::Room;
use crate::UnixTime;

pub fn newest_room_member(room: &Room) -> UnixTime {
    room.members
        .values()
        .map(|room_member| room_member.updated_at)
        .max()
        .unwrap_or(UnixTime::EPOCH)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::newest_room_member;
    use crate::Peer;
    use crate::PeerId;
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomSize;
    use crate::UnixTime;

    #[test]
    fn test_usage() {
        let member = |secs| RoomMember {
            peer: Peer {
                id: PeerId::from("peer"),
                addrs: Vec::new(),
            },
            updated_at: UnixTime::from_secs(secs),
            display_name: None,
            membership: RoomMembership::Occupant,
        };
        let room = Room {
            room_size: RoomSize::DEFAULT,
            created_at: UnixTime::EPOCH,
            members: BTreeMap::from([
                (PeerId::from("a"), member(5)),
                (PeerId::from("b"), member(9)),
            ]),
        };
        assert_eq!(newest_room_member(&room), UnixTime::from_secs(9));
        let empty = Room {
            members: BTreeMap::new(),
            ..room
        };
        assert_eq!(newest_room_member(&empty), UnixTime::EPOCH);
    }
}
