use crate::discovery;
use crate::testing;

#[must_use]
pub fn preview_rows_long() -> discovery::LobbyRows {
    let mut rows = vec![testing::lobby_row(
        "alpha",
        1,
        16,
        discovery::RoomOccupancy::HasSpace,
        0,
    )];
    for index in 2..=12_u8 {
        rows.push(testing::lobby_row(
            format!("room-{index:02}").as_str(),
            usize::from(index),
            16,
            discovery::RoomOccupancy::HasSpace,
            0,
        ));
    }
    discovery::LobbyRows(rows)
}

#[cfg(test)]
mod tests {
    use super::preview_rows_long;

    #[test]
    fn test_usage() {
        let rows = preview_rows_long();
        assert_eq!(rows.len(), 12);
        assert_eq!(rows[0].name.as_str(), "alpha");
        assert_eq!(rows[11].name.as_str(), "room-12");
    }
}
