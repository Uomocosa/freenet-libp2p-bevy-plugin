use crate::Lobby;

pub fn evict_oldest_rooms_until_lobby_fits_capacity(lobby: &mut Lobby, capacity: usize) {
    while lobby.len() > capacity {
        let oldest = lobby
            .iter()
            .min_by_key(|(name, room)| (room.newest_room_member(), name.as_str()))
            .map(|(name, _)| name.clone());
        let Some(oldest) = oldest else {
            break;
        };
        lobby.remove(&oldest);
    }
}

#[cfg(test)]
mod tests {
    use super::evict_oldest_rooms_until_lobby_fits_capacity;
    use crate::Lobby;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::testing;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let mut lobby = Lobby::from([
            (
                RoomName::from("old"),
                testing::room_with_one_member(
                    "peer",
                    Vec::new(),
                    UnixTime::from_secs(1),
                    RoomMembership::Occupant,
                ),
            ),
            (
                RoomName::from("new"),
                testing::room_with_one_member(
                    "peer",
                    Vec::new(),
                    UnixTime::from_secs(9),
                    RoomMembership::Occupant,
                ),
            ),
            (
                RoomName::from("mid"),
                testing::room_with_one_member(
                    "peer",
                    Vec::new(),
                    UnixTime::from_secs(5),
                    RoomMembership::Occupant,
                ),
            ),
        ]);
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&RoomName::from("old")));
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 5);
        assert_eq!(lobby.len(), 2);
    }
}
