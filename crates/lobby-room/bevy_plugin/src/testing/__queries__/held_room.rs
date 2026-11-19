use crate::discovery;

#[must_use]
pub fn held_room(
    state: &discovery::state_machine::DiscoveryState,
    room: &str,
) -> Option<discovery::state_machine::HeldRoomHistory> {
    state
        .held_rooms
        .get(&discovery::RoomName::from(room))
        .copied()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::held_room;

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[("alpha", 20)]))
            .build();
        assert_eq!(
            held_room(&state, "alpha"),
            Some(testing::held_room_history(0, 20))
        );
        assert_eq!(held_room(&state, "missing"), None);
    }
}
