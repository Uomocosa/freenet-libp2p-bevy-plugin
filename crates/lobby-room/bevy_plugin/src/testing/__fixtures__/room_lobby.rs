use crate::discovery;
use crate::testing;

#[must_use]
pub fn room_lobby() -> discovery::Lobby {
    testing::lobby_with(vec![(
        "room-a",
        testing::room(
            discovery::RoomSize::DEFAULT,
            0,
            vec![testing::room_member(
                "peer",
                None,
                discovery::RoomMembership::Occupant,
                &[],
            )],
        ),
    )])
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    use super::room_lobby;

    #[test]
    fn test_usage() {
        let lobby = room_lobby();
        assert_eq!(lobby.len(), 1);
        assert!(lobby.contains_key(&discovery::RoomName::from("room-a")));
    }
}
