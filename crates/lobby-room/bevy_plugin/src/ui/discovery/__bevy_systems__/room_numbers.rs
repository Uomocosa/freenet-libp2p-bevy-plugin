use crate::discovery as core_discovery;

#[must_use]
pub fn room_numbers(row: &core_discovery::LobbyRow) -> [u32; 3] {
    [
        u32::try_from(row.occupants).unwrap_or(u32::MAX),
        u32::from(row.room_size.get()),
        u32::try_from(row.friends).unwrap_or(u32::MAX),
    ]
}

#[cfg(test)]
mod tests {
    use super::room_numbers;
    use crate::discovery as core_discovery;

    fn row(occupants: usize, room_size: u16, friends: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants,
            room_size: core_discovery::RoomSize::try_from(room_size).expect("room size"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(room_numbers(&row(2, 8, 1)), [2, 8, 1]);
    }

    #[test]
    fn the_slots_are_occupants_room_size_friends() {
        assert_eq!(room_numbers(&row(0, 4, 12)), [0, 4, 12]);
    }
}
