use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{DiscoveryState, Output};

#[must_use]
pub fn send_handshake(state: &DiscoveryState, peer: &identity::PeerId) -> Vec<Output> {
    let data = bincode::serialize(&state.handshake()).unwrap_or_default();
    vec![Output::NetCommand(p2p::NetCommand::PeerExchange {
        peer_id: peer.clone(),
        data,
    })]
}

#[cfg(test)]
mod tests {
    use super::send_handshake;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{DiscoveryState, Output};

    #[test]
    fn test_usage() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        let outputs = send_handshake(&state, &identity::PeerId::from("a"));
        assert!(matches!(
            outputs.as_slice(),
            [Output::NetCommand(p2p::NetCommand::PeerExchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
    }
}
