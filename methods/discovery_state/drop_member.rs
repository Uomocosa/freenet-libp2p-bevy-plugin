use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Output};

pub fn drop_member(state: &mut DiscoveryState, peer: &identity::PeerId) -> Vec<Output> {
    let removed = state
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if !removed {
        return Vec::new();
    }
    tracing::info!(target: "discovery", peer = %peer.as_str(), "discovery member left");
    vec![Output::Notify(discovery::Event::MembersChanged)]
}

#[cfg(test)]
mod tests {
    use super::drop_member;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    fn state_in_room() -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            entry: discovery::RoomEntry::Created,
        });
        state
    }

    #[test]
    fn test_usage() {
        let mut state = state_in_room();
        let peer = identity::PeerId::from("a");
        assert_eq!(drop_member(&mut state, &peer), Vec::new());
        state.room.as_mut().expect("room").members.insert(
            peer.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer.clone(),
                    addrs: Vec::new(),
                },
                updated_at: discovery::UnixTime::from(1),
                display_name: None,
                membership: discovery::RoomMembership::Occupant,
            },
        );
        let outputs = drop_member(&mut state, &peer);
        assert_eq!(outputs.len(), 1);
        assert!(
            state
                .room
                .as_ref()
                .is_none_or(|room| room.members.is_empty())
        );
    }
}
