use crate::discovery;
use crate::testing;

#[must_use]
pub fn room_members(
    members: &[(&str, Option<&str>, discovery::RoomMembership)],
) -> Vec<discovery::RoomMember> {
    members
        .iter()
        .copied()
        .map(|(peer, display_name, membership)| {
            testing::room_member(peer, display_name, membership, &[])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;

    use super::room_members;

    #[test]
    fn test_usage() {
        let members = room_members(&[
            ("a", None, discovery::RoomMembership::Occupant),
            ("b", Some("Bob"), discovery::RoomMembership::Departed),
        ]);
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].peer.id.as_str(), "a");
        assert_eq!(
            members[1].display_name,
            Some(identity::Username::from("Bob"))
        );
        assert_eq!(members[1].membership, discovery::RoomMembership::Departed);
    }
}
