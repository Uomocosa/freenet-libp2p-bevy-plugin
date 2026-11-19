use common_types::PeerAddr;
use common_types::UnixTime;

use crate::Room;
use crate::RoomMembership;
use crate::testing;

#[must_use]
pub fn room_with_one_member(
    id: &str,
    addrs: Vec<PeerAddr>,
    updated_at: UnixTime,
    membership: RoomMembership,
) -> Room {
    testing::room_with_members(vec![testing::room_member(
        id, addrs, updated_at, membership,
    )])
}

#[cfg(test)]
mod tests {
    use super::room_with_one_member;
    use crate::RoomMembership;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let room = room_with_one_member(
            "peer",
            Vec::new(),
            UnixTime::from_secs(5),
            RoomMembership::Departed,
        );
        assert_eq!(room.members.len(), 1);
        let member = room.members.values().next().expect("one member");
        assert_eq!(member.updated_at, UnixTime::from_secs(5));
        assert_eq!(member.membership, RoomMembership::Departed);
    }
}
