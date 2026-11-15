use crate::discovery;
use crate::p2p;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

#[must_use]
pub fn tick(state: &mut DiscoveryState, now: time::UnixTime) -> Vec<Output> {
    let mut outputs = state.dial_candidates(now);
    if state.next_handshake.is_due(now) && state.room.is_some() {
        outputs.extend(state.broadcast_handshake());
        state.next_handshake =
            time::Deadline::from_now_plus(now, discovery::state_machine::HANDSHAKE_EVERY);
    }
    state.drop_expired_candidates(now);
    outputs.extend(state.drop_members_past_grace(now));
    outputs.extend(state.drop_lobby_members_whose_lease_lapsed(now));
    outputs.extend(state.wait_for_contract_self_heal(now));
    outputs.push(Output::Libp2pCommand(
        p2p::Libp2pCommand::MeasureHistoryStore,
    ));
    outputs
}
#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;
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
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        let mut outputs = tick(&mut state, time::UnixTime::from_secs(100));
        outputs.extend(tick(&mut state, time::UnixTime::from_secs(101)));
        assert_eq!(
            state.next_handshake,
            time::Deadline::from_now_plus(
                time::UnixTime::from_secs(100),
                discovery::state_machine::HANDSHAKE_EVERY
            )
        );
        let measures = outputs
            .iter()
            .filter(|output| {
                **output
                    == discovery::state_machine::Output::Libp2pCommand(
                        p2p::Libp2pCommand::MeasureHistoryStore,
                    )
            })
            .count();
        assert_eq!((outputs.len(), measures), (3, 2));
    }
}
