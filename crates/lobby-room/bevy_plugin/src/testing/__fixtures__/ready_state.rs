use crate::discovery;
use crate::identity;
use crate::testing;

#[must_use]
pub fn ready_state(
    me: &str,
    room: &str,
    display_name: Option<&str>,
    members: Vec<discovery::RoomMember>,
    connected: &[&str],
) -> discovery::state_machine::DiscoveryState {
    testing::DiscoveryStateBuilder::new(identity::PeerId::from(me))
        .with_display_name(identity::Username::from(display_name.unwrap_or("")))
        .in_room(testing::joined_room(
            room,
            1,
            discovery::JoinPhase::Ready,
            members,
        ))
        .with_connected(
            connected
                .iter()
                .map(|peer| identity::PeerId::from(*peer))
                .collect(),
        )
        .build()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    use super::ready_state;

    #[test]
    fn test_usage() {
        let state = ready_state(
            "me",
            "alpha",
            Some("Me"),
            vec![testing::room_member(
                "other",
                Some("Other"),
                discovery::RoomMembership::Occupant,
                &[],
            )],
            &["other"],
        );
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        assert_eq!(state.display_name, identity::Username::from("Me"));
        let room = state.room.as_ref().expect("a joined room");
        assert_eq!(room.name, discovery::RoomName::from("alpha"));
        assert_eq!(room.phase, discovery::JoinPhase::Ready);
        assert!(state.connected.contains(&identity::PeerId::from("other")));
    }
}
