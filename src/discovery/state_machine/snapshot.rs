use crate::discovery;
use discovery::state_machine::State;

#[must_use]
pub fn snapshot(state: &State) -> discovery::Snapshot {
    discovery::Snapshot {
        lobby: state.lobby.clone(),
        room: state.room.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::snapshot;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        assert_eq!(snapshot(&state), discovery::Snapshot::default());
    }
}
