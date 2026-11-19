use common_types::UnixTime;

use crate::Room;
use crate::RoomMembership;
use crate::testing;

#[must_use]
pub fn departed_room(updated_at: UnixTime) -> Room {
    let mut room = testing::room("peer", updated_at, UnixTime::from_secs(1));
    for heartbeat in room.members.values_mut() {
        heartbeat.membership = RoomMembership::Departed;
    }
    room
}

#[cfg(test)]
mod tests {
    use super::departed_room;
    use crate::RoomMembership;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let room = departed_room(UnixTime::from_secs(9));
        assert!(
            room.members
                .values()
                .all(|member| member.membership == RoomMembership::Departed)
        );
    }
}
