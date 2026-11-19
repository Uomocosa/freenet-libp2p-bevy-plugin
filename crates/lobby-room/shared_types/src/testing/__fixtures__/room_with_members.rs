use std::collections::BTreeMap;

use common_types::UnixTime;

use crate::Room;
use crate::RoomMember;
use crate::RoomSize;

#[must_use]
pub fn room_with_members(members: Vec<RoomMember>) -> Room {
    Room {
        room_size: RoomSize::DEFAULT,
        created_at: UnixTime::EPOCH,
        members: members
            .into_iter()
            .map(|member| (member.peer.id.clone(), member))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[cfg(test)]
mod tests {
    use super::room_with_members;
    use crate::RoomMembership;
    use crate::testing;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let room = room_with_members(vec![
            testing::room_member("a", Vec::new(), UnixTime::EPOCH, RoomMembership::Occupant),
            testing::room_member(
                "b",
                Vec::new(),
                UnixTime::from_secs(1),
                RoomMembership::HistoryHolder,
            ),
        ]);
        assert_eq!(room.members.len(), 2);
        assert_eq!(room.created_at, UnixTime::EPOCH);
    }
}
