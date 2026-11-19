use crate::discovery;
use crate::testing;

#[must_use]
pub fn lobby_row(
    name: &str,
    occupants: usize,
    room_size: u16,
    occupancy: discovery::RoomOccupancy,
    friends: usize,
) -> discovery::LobbyRow {
    discovery::LobbyRow {
        name: discovery::RoomName::from(name),
        occupants,
        room_size: testing::room_size(room_size),
        occupancy,
        friends,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    use super::lobby_row;

    #[test]
    fn test_usage() {
        let row = lobby_row("alpha", 2, 8, discovery::RoomOccupancy::HasSpace, 1);
        assert_eq!(row.name, discovery::RoomName::from("alpha"));
        assert_eq!(row.occupants, 2);
        assert_eq!(row.room_size.get(), 8);
        assert_eq!(row.occupancy, discovery::RoomOccupancy::HasSpace);
        assert_eq!(row.friends, 1);
    }
}
