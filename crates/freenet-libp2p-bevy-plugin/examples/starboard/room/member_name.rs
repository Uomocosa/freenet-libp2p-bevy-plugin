use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::{discovery, identity};

#[must_use]
pub fn member_name(state: &DiscoveryState, peer: &identity::PeerId) -> String {
    state
        .room
        .as_ref()
        .and_then(|room| room.members.get(peer))
        .and_then(|member| member.display_name.as_ref())
        .map_or_else(|| peer.as_str().to_string(), ToString::to_string)
}

#[cfg(test)]
mod tests {
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::testing;
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::member_name;

    #[test]
    fn test_usage() {
        let peer = identity::PeerId::from("peer-a");
        let mut members = discovery::Members::new();
        members.insert(
            peer.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer.clone(),
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::EPOCH,
                display_name: Some(identity::Username::from("ada")),
                membership: discovery::RoomMembership::Occupant,
            },
        );
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members,
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build();
        assert_eq!(member_name(&state, &peer), "ada");
        let stranger = identity::PeerId::from("peer-z");
        assert_eq!(member_name(&state, &stranger), "peer-z");
    }
}
