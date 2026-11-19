use crate::discovery;

#[must_use]
pub const fn room_roster(
    own: discovery::OwnRoomStatus,
    count: usize,
    max: usize,
    entries: Vec<discovery::RoomRosterEntry>,
) -> discovery::RoomRoster {
    discovery::RoomRoster {
        own,
        count,
        max,
        entries,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;

    use super::room_roster;

    #[test]
    fn test_usage() {
        let roster = room_roster(
            discovery::OwnRoomStatus::Connected,
            1,
            8,
            vec![testing::room_roster_entry(
                "me",
                "Me",
                discovery::MemberRoomStatus::Connected,
                true,
            )],
        );
        assert_eq!(roster.own, discovery::OwnRoomStatus::Connected);
        assert_eq!(roster.count, 1);
        assert_eq!(roster.max, 8);
        assert_eq!(roster.entries.len(), 1);
    }
}
