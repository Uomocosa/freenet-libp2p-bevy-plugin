use crate::discovery;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn prune_members(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let ttl = state.timing.presence_ttl;
    let grace = state.timing.member_grace;
    state
        .candidates
        .retain(|_, presence| now.since(presence.updated_at) <= ttl);
    let connected = &state.connected;
    let Some(room) = state.room.as_mut() else {
        return Vec::new();
    };
    let before = room.members.len();
    room.members.retain(|peer, member| {
        connected.contains(peer) || now.since(member.presence.updated_at) <= grace
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
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        let mut members = discovery::Members::new();
        for peer in ["live", "gone"] {
            members.insert(
                net_id::PeerId(peer.to_string()),
                discovery::Member {
                    presence: discovery::Presence {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                    },
                    status: discovery::LinkStatus::Known,
                },
            );
        }
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members,
        });
        state.connected.insert(net_id::PeerId::from("live"));
        let _ = prune_members(&mut state, discovery::UnixTime::from_secs(100));
        let left: Vec<_> = state
            .room
            .as_ref()
            .map(|room| room.members.keys().cloned().collect())
            .unwrap_or_default();
        assert_eq!(left, vec![net_id::PeerId::from("live")]);
    }
}
