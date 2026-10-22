use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Output};

#[must_use]
pub fn broadcast_handshake(state: &DiscoveryState) -> Vec<Output> {
    let peers: Vec<identity::PeerId> = state.connected.iter().cloned().collect();
    peers
        .iter()
        .flat_map(|peer| state.send_handshake(peer))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::broadcast_handshake;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        state.connected.insert(identity::PeerId::from("a"));
        state.connected.insert(identity::PeerId::from("b"));
        assert_eq!(broadcast_handshake(&state).len(), 2);
    }
}
