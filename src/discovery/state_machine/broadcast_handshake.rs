use crate::discovery;
use crate::identity;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn broadcast_handshake(state: &State) -> Vec<Output> {
    let peers: Vec<identity::PeerId> = state.connected.iter().cloned().collect();
    peers
        .iter()
        .flat_map(|peer| discovery::state_machine::send_handshake(state, peer))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::broadcast_handshake;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        state.connected.insert(identity::PeerId::from("a"));
        state.connected.insert(identity::PeerId::from("b"));
        assert_eq!(broadcast_handshake(&state).len(), 2);
    }
}
