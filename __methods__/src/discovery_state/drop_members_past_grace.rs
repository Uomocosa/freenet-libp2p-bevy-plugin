use crate::discovery;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

pub fn drop_members_past_grace(state: &mut DiscoveryState, now: time::UnixTime) -> Vec<Output> {
    let grace = discovery::state_machine::MEMBER_GRACE;
    let connected = &state.connected;
    let Some(room) = state.room.as_mut() else {
        return Vec::new();
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        let should_keep = connected.contains(peer) || now.elapsed_since(member.updated_at) <= grace;
        if !should_keep {
            tracing::info!(
                target: "discovery",
                peer = %peer.as_str(),
                "discovery dropped member past grace"
            );
        }
        should_keep
    });
    if room.members.len() == before {
        Vec::new()
    } else {
        vec![Output::Notify(discovery::Event::RoomMembersUpdated)]
    }
}
#[cfg(test)]
mod tests {
    use super::drop_members_past_grace;
    use crate::discovery;
    use crate::identity;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let mut members = discovery::Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(peer),
                        addrs: Vec::new(),
                    },
                    updated_at: time::UnixTime::from(0),
                    display_name: None,
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: time::UnixTime::from(0),
            phase: discovery::JoinPhase::Ready,
            members,
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        state.connected.insert(identity::PeerId::from("live"));
        let _ = drop_members_past_grace(&mut state, time::UnixTime::from(100));
        let left: Vec<_> = state
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![identity::PeerId::from("live")]);
    }
}
