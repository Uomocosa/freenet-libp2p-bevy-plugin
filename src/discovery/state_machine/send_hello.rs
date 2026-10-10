use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn send_hello(state: &State, peer: &net_id::PeerId) -> Vec<Output> {
    let data = bincode::serialize(&discovery::state_machine::hello(state)).unwrap_or_default();
    vec![Output::NetCommand(p2p::NetCommand::Exchange {
        peer_id: peer.clone(),
        data,
    })]
}

#[cfg(test)]
mod tests {
    use super::send_hello;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        let outputs = send_hello(&state, &net_id::PeerId::from("a"));
        assert!(matches!(
            outputs.as_slice(),
            [Output::NetCommand(p2p::NetCommand::Exchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
    }
}
