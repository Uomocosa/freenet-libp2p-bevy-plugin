use crate::discovery;
use crate::time;

#[must_use]
pub fn room(
    room_size: discovery::RoomSize,
    created_at: u64,
    members: Vec<discovery::RoomMember>,
) -> discovery::Room {
    discovery::Room {
        room_size,
        created_at: time::UnixTime::from_secs(created_at),
        members: members
            .into_iter()
            .map(|member| (member.peer.id.clone(), member))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    use super::room;

    #[test]
    fn test_usage() {
        let fixture = room(
            discovery::RoomSize::DEFAULT,
            7,
            vec![testing::room_member(
                "a",
                None,
                discovery::RoomMembership::Occupant,
                &[],
            )],
        );
        assert_eq!(fixture.room_size, discovery::RoomSize::DEFAULT);
        assert_eq!(fixture.created_at, time::UnixTime::from_secs(7));
        assert_eq!(fixture.members.len(), 1);
        assert!(fixture.members.contains_key(&identity::PeerId::from("a")));
    }
}
