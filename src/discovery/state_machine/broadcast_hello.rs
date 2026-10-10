use crate::discovery;
use crate::net_id;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn broadcast_hello(state: &State) -> Vec<Output> {
    let peers: Vec<net_id::PeerId> = state.connected.iter().cloned().collect();
    peers
        .iter()
        .flat_map(|peer| discovery::state_machine::send_hello(state, peer))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::broadcast_hello;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        state.connected.insert(net_id::PeerId::from("b"));
        assert_eq!(broadcast_hello(&state).len(), 2);
    }
}
