use crate::discovery as core_discovery;

#[must_use]
pub fn occupancy_tooltip_text(row: &core_discovery::LobbyRow) -> String {
    let players = format!("{} of {} players", row.occupants, row.room_size);
    match row.friends {
        0 => players,
        1 => format!("{players} | 1 friend"),
        friends => format!("{players} | {friends} friends"),
    }
}

#[cfg(test)]
mod tests {
    use super::occupancy_tooltip_text;
    use crate::discovery as core_discovery;

    fn row_with_friends(friends: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 3,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            occupancy_tooltip_text(&row_with_friends(0)),
            "3 of 4 players"
        );
    }

    #[test]
    fn the_tooltip_counts_players_without_a_friend_suffix_when_there_are_no_friends() {
        assert_eq!(
            occupancy_tooltip_text(&row_with_friends(0)),
            "3 of 4 players"
        );
    }

    #[test]
    fn the_tooltip_uses_the_singular_friend_word_for_one_friend() {
        assert_eq!(
            occupancy_tooltip_text(&row_with_friends(1)),
            "3 of 4 players | 1 friend"
        );
    }

    #[test]
    fn the_tooltip_uses_the_plural_friend_word_for_two_friends() {
        assert_eq!(
            occupancy_tooltip_text(&row_with_friends(2)),
            "3 of 4 players | 2 friends"
        );
    }
}
