use crate::discovery;
use discovery::state_machine::State;

#[must_use]
pub fn status(state: &State) -> discovery::Status {
    discovery::Status {
        lobby: state.lobby.clone(),
        room: state.room.clone(),
        freenet_peers: state.freenet_peers.clone(),
        is_lobby_attached: state.is_lobby_attached,
        self_peer_id: Some(state.me.id.clone()),
        lease_ttl: state.lease_ttl,
    }
}

#[cfg(test)]
mod tests {
    use super::status;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        let expected = discovery::Status {
            self_peer_id: Some(identity::PeerId::from("me")),
            ..discovery::Status::default()
        };
        assert_eq!(status(&state), expected);
    }
}
