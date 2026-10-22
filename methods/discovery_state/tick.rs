use crate::discovery;
use crate::p2p;
use discovery::state_machine::{DiscoveryState, Output};

#[must_use]
pub fn tick(state: &mut DiscoveryState, now: discovery::UnixTime) -> Vec<Output> {
    let mut outputs = state.dial_candidates(now);
    let handshake_due = state
        .last_handshake_at
        .is_none_or(|last| now.elapsed_since(last) >= discovery::state_machine::HANDSHAKE_EVERY);
    if handshake_due && state.room.is_some() {
        outputs.extend(state.broadcast_handshake());
        state.last_handshake_at = Some(now);
    }
    outputs.extend(state.prune_members(now));
    outputs.extend(state.wait_for_contract_self_heal(now));
    outputs.push(Output::NetCommand(p2p::NetCommand::MeasureHistoryStore));
    outputs
}

#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.connected.insert(identity::PeerId::from("a"));
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
        });
        let mut outputs = tick(&mut state, discovery::UnixTime::from_secs(100));
        outputs.extend(tick(&mut state, discovery::UnixTime::from_secs(101)));
        assert_eq!(
            state.last_handshake_at,
            Some(discovery::UnixTime::from_secs(100))
        );
        let measures = outputs
            .iter()
            .filter(|output| {
                **output
                    == discovery::state_machine::Output::NetCommand(
                        p2p::NetCommand::MeasureHistoryStore,
                    )
            })
            .count();
        assert_eq!((outputs.len(), measures), (3, 2));
    }
}
