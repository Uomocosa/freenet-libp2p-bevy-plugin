use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;

#[must_use]
pub fn member_name(
    state: &DiscoveryState,
    zone: Option<&identity::ZoneName>,
    peer: &identity::PeerId,
) -> String {
    let members = zone.map_or_else(
        || state.room.as_ref().map(|room| &room.members),
        |zone| {
            state
                .lobby
                .get(&discovery::RoomName::from(zone.clone()))
                .map(|room| &room.members)
        },
    );
    members
        .and_then(|members| members.get(peer))
        .and_then(|member| member.display_name.as_ref())
        .map_or_else(|| peer.as_str().to_string(), ToString::to_string)
}

#[cfg(test)]
mod tests {

    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;
    use freenet_libp2p_bevy_plugin::time;

    use super::member_name;

    fn member(peer: &str, name: &str) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: Some(identity::Username::from(name)),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    fn status_with_room_and_lobby(peer: &str, name: &str) -> DiscoveryState {
        let mut members = discovery::Members::new();
        members.insert(identity::PeerId::from(peer), member(peer, name));
        let lobby = discovery::Lobby::from([(
            discovery::RoomName::from("beta"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: members.clone(),
            },
        )]);
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .in_room(discovery::JoinedRoom {
            name: discovery::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members,
            how_entered: discovery::HowRoomWasEntered::Created,
        })
        .build()
    }

    #[test]
    fn test_usage() {
        let peer = identity::PeerId::from("peer-a");
        let state = status_with_room_and_lobby("peer-a", "ada");
        assert_eq!(member_name(&state, None, &peer), "ada");
        let beta = identity::ZoneName::from("beta");
        assert_eq!(member_name(&state, Some(&beta), &peer), "ada");
        let stranger = identity::PeerId::from("peer-z");
        assert_eq!(member_name(&state, None, &stranger), "peer-z");
    }

    #[test]
    fn a_zone_name_comes_from_the_lobby_record_not_the_joined_room() {
        let peer = identity::PeerId::from("peer-a");
        let state = status_with_room_and_lobby("peer-a", "ada");
        let gamma = identity::ZoneName::from("gamma");
        assert_eq!(
            member_name(&state, Some(&gamma), &peer),
            "peer-a",
            "an unknown zone falls back to the peer id"
        );
    }
}
