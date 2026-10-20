use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{Output, State};

pub fn handle_net_event(
    state: &mut State,
    event: p2p::NetEvent,
    now: discovery::UnixTime,
) -> Vec<Output> {
    let mut outputs = match event {
        p2p::NetEvent::PeerConnected(peer) => {
            state.connected.insert(peer.clone());
            discovery::state_machine::send_handshake(state, &peer)
        }
        p2p::NetEvent::PeerDisconnected(peer) => {
            state.connected.remove(&peer);
            mark_lost(state, &peer, now)
        }
        p2p::NetEvent::PeerExchange { from, data } => {
            discovery::state_machine::handle_handshake(state, &from, &data, now)
        }
        p2p::NetEvent::HistoryRecordStored { .. } => {
            return vec![Output::NetCommand(p2p::NetCommand::MeasureHistoryStore)];
        }
        p2p::NetEvent::HistoryStoreMeasured { bytes_by_prefix } => {
            return discovery::state_machine::drop_oldest_history(state, &bytes_by_prefix);
        }
        _ => return Vec::new(),
    };
    outputs.extend(discovery::state_machine::dial_candidates(state, now));
    outputs
}

// needed helper: keeps a dropped member for the grace window, marked not connected
fn mark_lost(state: &mut State, peer: &identity::PeerId, now: discovery::UnixTime) -> Vec<Output> {
    let Some(member) = state
        .room
        .as_mut()
        .and_then(|room| room.members.get_mut(peer))
    else {
        return Vec::new();
    };
    member.status = discovery::LinkStatus::Lost;
    member.heartbeat.updated_at = now;
    vec![Output::Notify(discovery::Event::MembersChanged)]
}

#[cfg(test)]
mod tests {
    use super::handle_net_event;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs = handle_net_event(
            &mut state,
            p2p::NetEvent::PeerConnected(identity::PeerId::from("a")),
            discovery::UnixTime::from_secs(1),
        );
        assert!(state.connected.contains(&identity::PeerId::from("a")));
        assert!(matches!(
            outputs.as_slice(),
            [Output::NetCommand(p2p::NetCommand::PeerExchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
        let _ = handle_net_event(
            &mut state,
            p2p::NetEvent::PeerDisconnected(identity::PeerId::from("a")),
            discovery::UnixTime::from_secs(2),
        );
        assert!(state.connected.is_empty());
    }
}
