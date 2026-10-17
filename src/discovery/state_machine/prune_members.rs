use crate::discovery;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn prune_members(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let ttl = discovery::state_machine::HEARTBEAT_TTL;
    let grace = discovery::state_machine::MEMBER_GRACE;
    state
        .candidates
        .retain(|_, candidate| now.since(candidate.seen_at) <= ttl);
    let connected = &state.connected;
    let Some(room) = state.room.as_mut() else {
        return Vec::new();
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        connected.contains(peer) || now.since(member.heartbeat.updated_at) <= grace
    });
    if room.members.len() == before {
        Vec::new()
    } else {
        vec![Output::Notify(discovery::Event::MembersChanged)]
    }
}

#[cfg(test)]
mod tests {
    use super::prune_members;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let mut members = discovery::Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: None,
                    },
                    status: discovery::LinkStatus::Lost,
                },
            );
        }
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            members,
        });
        state.connected.insert(identity::PeerId::from("live"));
        let _ = prune_members(&mut state, discovery::UnixTime::from_secs(100));
        let left: Vec<_> = state
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![identity::PeerId::from("live")]);
    }
}
