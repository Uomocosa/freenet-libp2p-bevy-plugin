use crate::Lobby;
use crate::RoomMembership;

pub fn drop_rooms_where_every_member_quit(lobby: &mut Lobby) {
    lobby.retain(|_, room| {
        room.members
            .values()
            .any(|room_member| room_member.membership != RoomMembership::Departed)
    });
}

#[cfg(test)]
mod tests {
    use super::drop_rooms_where_every_member_quit;
    use crate::Lobby;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::testing;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let mut lobby = Lobby::from([
            (
                RoomName::from("gone"),
                testing::room_with_one_member(
                    "peer",
                    Vec::new(),
                    UnixTime::from_secs(1),
                    RoomMembership::Departed,
                ),
            ),
            (
                RoomName::from("live"),
                testing::room_with_one_member(
                    "peer",
                    Vec::new(),
                    UnixTime::from_secs(1),
                    RoomMembership::Occupant,
                ),
            ),
        ]);
        drop_rooms_where_every_member_quit(&mut lobby);
        assert_eq!(lobby.len(), 1);
        assert!(lobby.contains_key(&RoomName::from("live")));
    }
}
