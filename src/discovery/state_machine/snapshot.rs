use crate::discovery;
use discovery::state_machine::State;

#[must_use]
pub fn snapshot(state: &State) -> discovery::Snapshot {
    discovery::Snapshot {
        lobby: state.lobby.clone(),
        room: state.room.clone(),
        freenet_peers: state.freenet_peers.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::snapshot;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let state = State::new(
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: vec![identity::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        assert_eq!(snapshot(&state), discovery::Snapshot::default());
    }
}
