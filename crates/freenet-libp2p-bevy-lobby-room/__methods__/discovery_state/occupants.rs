use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;
use std::collections::BTreeSet;

pub fn occupants(state: &DiscoveryState) -> Vec<identity::PeerId> {
    let Some(room) = state.room.as_ref() else {
        return Vec::new();
    };
    let mut present = BTreeSet::new();
    present.insert(state.me.id.clone());
    for (peer, member) in &room.members {
        if member.membership == discovery::RoomMembership::Occupant {
            present.insert(peer.clone());
        }
    }
    present.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::occupants;
    use crate::discovery;
    use crate::identity;
    use crate::time;
    use discovery::state_machine::DiscoveryState;
    use std::collections::BTreeMap;

    fn room_member(
        peer: &identity::PeerId,
        membership: discovery::RoomMembership,
    ) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: peer.clone(),
                addrs: Vec::new(),
            },
            updated_at: time::UnixTime::EPOCH,
            display_name: None,
            membership,
        }
    }

    fn ready_state(me: &identity::PeerId, members: discovery::Members) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: me.clone(),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::from_secs(1),
            phase: discovery::JoinPhase::Ready,
            members,
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        state
    }

    #[test]
    fn test_usage() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert_eq!(occupants(&state), Vec::<identity::PeerId>::new());
    }

    #[test]
    fn test_usage_1() {
        let mut members = discovery::Members::new();
        for (peer, membership) in [
            ("occupant", discovery::RoomMembership::Occupant),
            ("holder", discovery::RoomMembership::HistoryHolder),
            ("departed", discovery::RoomMembership::Departed),
        ] {
            members.insert(
                identity::PeerId::from(peer),
                room_member(&identity::PeerId::from(peer), membership),
            );
        }
        let state = ready_state(&identity::PeerId::from("me"), members);
        assert_eq!(
            occupants(&state),
            vec![
                identity::PeerId::from("me"),
                identity::PeerId::from("occupant"),
            ]
        );
    }

    #[test]
    fn test_usage_2() {
        let mut members = BTreeMap::new();
        for peer in ["me", "other"] {
            members.insert(
                identity::PeerId::from(peer),
                room_member(
                    &identity::PeerId::from(peer),
                    discovery::RoomMembership::Occupant,
                ),
            );
        }
        let state = ready_state(&identity::PeerId::from("me"), members);
        assert_eq!(
            occupants(&state),
            vec![
                identity::PeerId::from("me"),
                identity::PeerId::from("other"),
            ]
        );
    }
}
