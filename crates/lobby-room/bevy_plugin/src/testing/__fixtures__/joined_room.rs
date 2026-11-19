use crate::discovery;
use crate::time;

#[must_use]
pub fn joined_room(
    name: &str,
    created_at: u64,
    phase: discovery::JoinPhase,
    members: Vec<discovery::RoomMember>,
) -> discovery::JoinedRoom {
    discovery::JoinedRoom {
        name: discovery::RoomName::from(name),
        created_at: time::UnixTime::from_secs(created_at),
        phase,
        members: members
            .into_iter()
            .map(|member| (member.peer.id.clone(), member))
            .collect(),
        how_entered: discovery::HowRoomWasEntered::Created,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    use super::joined_room;

    #[test]
    fn test_usage() {
        let fixture = joined_room(
            "alpha",
            3,
            discovery::JoinPhase::Ready,
            vec![testing::room_member(
                "a",
                None,
                discovery::RoomMembership::Occupant,
                &[],
            )],
        );
        assert_eq!(fixture.name, discovery::RoomName::from("alpha"));
        assert_eq!(fixture.created_at, time::UnixTime::from_secs(3));
        assert_eq!(fixture.phase, discovery::JoinPhase::Ready);
        assert_eq!(fixture.how_entered, discovery::HowRoomWasEntered::Created);
        assert!(fixture.members.contains_key(&identity::PeerId::from("a")));
    }
}
