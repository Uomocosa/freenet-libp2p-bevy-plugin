use common_types::PeerAddr;
use common_types::UnixTime;

use crate::Room;
use crate::RoomMembership;
use crate::testing;

const MEMBER_ADDR: &str = "/ip4/127.0.0.1/tcp/4001";

#[must_use]
pub fn room_with_history_holders_and_occupants() -> Room {
    let members = [
        ("member-a", RoomMembership::HistoryHolder),
        ("member-b", RoomMembership::Occupant),
        ("member-c", RoomMembership::HistoryHolder),
        ("member-d", RoomMembership::Occupant),
        ("member-e", RoomMembership::HistoryHolder),
    ]
    .into_iter()
    .map(|(id, membership)| {
        testing::room_member(
            id,
            vec![PeerAddr::from(MEMBER_ADDR)],
            UnixTime::EPOCH,
            membership,
        )
    })
    .collect();
    testing::room_with_members(members)
}

#[cfg(test)]
mod tests {
    use super::room_with_history_holders_and_occupants;
    use crate::RoomMembership;

    #[test]
    fn test_usage() {
        let room = room_with_history_holders_and_occupants();
        assert_eq!(room.members.len(), 5);
        let history_holders = room
            .members
            .values()
            .filter(|member| member.membership == RoomMembership::HistoryHolder)
            .count();
        assert_eq!(history_holders, 3);
    }
}
