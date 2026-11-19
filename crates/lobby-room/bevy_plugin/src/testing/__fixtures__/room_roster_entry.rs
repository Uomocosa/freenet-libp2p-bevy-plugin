use crate::discovery;
use crate::identity;

#[must_use]
pub fn room_roster_entry(
    id: &str,
    name: &str,
    status: discovery::MemberRoomStatus,
    is_me: bool,
) -> discovery::RoomRosterEntry {
    discovery::RoomRosterEntry {
        id: identity::PeerId::from(id),
        name: String::from(name),
        status,
        is_me,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;

    use super::room_roster_entry;

    #[test]
    fn test_usage() {
        let entry = room_roster_entry("far", "Far", discovery::MemberRoomStatus::Connecting, false);
        assert_eq!(entry.id, identity::PeerId::from("far"));
        assert_eq!(entry.name, "Far");
        assert_eq!(entry.status, discovery::MemberRoomStatus::Connecting);
        assert!(!entry.is_me);
    }
}
