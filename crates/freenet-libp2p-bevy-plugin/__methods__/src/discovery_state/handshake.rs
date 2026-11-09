use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Handshake};

#[must_use]
pub fn handshake(state: &DiscoveryState) -> Handshake {
    let Some(room) = &state.room else {
        return Handshake {
            room: None,
            addrs: state.me.addrs.clone(),
            peers: Vec::new(),
            display_name: state.display_name.clone(),
        };
    };
    let peers = room
        .members
        .iter()
        .map(|(peer, member)| identity::Peer {
            id: peer.clone(),
            addrs: member.peer.addrs.clone(),
        })
        .collect();
    Handshake {
        room: Some(room.name.clone()),
        addrs: state.me.addrs.clone(),
        peers,
        display_name: state.display_name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::handshake;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        let handshake = handshake(&state);
        assert_eq!(handshake.room, None);
        assert_eq!(handshake.addrs, vec![identity::PeerAddr::from("/ip4/9")]);
    }
}
