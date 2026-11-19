use crate::RoomMember;

#[must_use]
pub fn picked_id(pick: Option<&RoomMember>) -> &str {
    pick.expect("a member is picked").peer.id.as_str()
}

#[cfg(test)]
mod tests {
    use super::picked_id;
    use crate::RoomMembership;
    use crate::testing;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let member = testing::room_member(
            "member-a",
            Vec::new(),
            UnixTime::EPOCH,
            RoomMembership::Occupant,
        );
        assert_eq!(picked_id(Some(&member)), "member-a");
    }

    #[test]
    #[should_panic(expected = "a member is picked")]
    fn panics_without_a_pick() {
        let _ = picked_id(None);
    }
}
