use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;

#[must_use]
pub fn is_in_room(state: &DiscoveryState) -> bool {
    state
        .room
        .as_ref()
        .is_some_and(|room| room.phase == discovery::JoinPhase::Ready)
}

#[cfg(test)]
mod tests {
    use discovery::state_machine::DiscoveryState;

    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::discovery;
    use freenet_libp2p_bevy_plugin::time;

    use super::is_in_room;

    fn status_with(phase: discovery::JoinPhase) -> DiscoveryState {
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .in_room(discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        })
        .build()
    }

    fn empty_state() -> DiscoveryState {
        freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .build()
    }

    #[test]
    fn test_usage() {
        assert!(!is_in_room(&empty_state()));
        assert!(!is_in_room(&status_with(discovery::JoinPhase::Joining {
            since: time::UnixTime::EPOCH
        })));
        assert!(is_in_room(&status_with(discovery::JoinPhase::Ready)));
    }
}
