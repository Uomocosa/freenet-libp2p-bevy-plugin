use crate::discovery;
use crate::testing;

#[must_use]
pub fn preview_rows() -> discovery::LobbyRows {
    discovery::LobbyRows(vec![
        testing::lobby_row("alpha", 2, 8, discovery::RoomOccupancy::HasSpace, 2),
        testing::lobby_row("beta", 0, 4, discovery::RoomOccupancy::Empty, 0),
        testing::lobby_row("gamma", 4, 4, discovery::RoomOccupancy::Full, 0),
    ])
}

#[cfg(test)]
mod tests {
    use super::preview_rows;

    #[test]
    fn test_usage() {
        assert_eq!(preview_rows().len(), 3);
    }
}
