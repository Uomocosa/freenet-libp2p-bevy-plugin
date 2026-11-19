use std::collections::BTreeMap;

use crate::discovery;
use crate::testing;

#[must_use]
pub fn held_rooms_by_room(
    rooms: &[(&str, u64)],
) -> BTreeMap<discovery::RoomName, discovery::state_machine::HeldRoomHistory> {
    rooms
        .iter()
        .copied()
        .map(|(room, left_at)| {
            (
                discovery::RoomName::from(room),
                testing::held_room_history(0, left_at),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::time;

    use super::held_rooms_by_room;

    #[test]
    fn test_usage() {
        let held = held_rooms_by_room(&[("alpha", 20)]);
        let history = held
            .get(&discovery::RoomName::from("alpha"))
            .copied()
            .expect("alpha is held");
        assert_eq!(history.created_at, time::UnixTime::EPOCH);
        assert_eq!(history.left_at, time::UnixTime::from_secs(20));
    }
}
