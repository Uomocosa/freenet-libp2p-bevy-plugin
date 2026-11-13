use crate::direct_dial;
use crate::identity;
use crate::time;

pub fn handle_peer_listen_addrs(
    state: &mut direct_dial::state_machine::DialState,
    peer: &identity::Peer,
    now: time::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    if peer.addrs.is_empty() {
        return Vec::new();
    }
    if let Some(entry) = state
        .entries
        .iter_mut()
        .find(|entry| entry.peer.id == peer.id)
    {
        if entry.peer.addrs == peer.addrs {
            return Vec::new();
        }
        entry.peer.addrs.clone_from(&peer.addrs);
        return vec![direct_dial::state_machine::Output::SavePeerBook(
            state.to_peer_book(),
        )];
    }
    if !state.connected.contains(&peer.id) {
        return Vec::new();
    }
    state
        .entries
        .push(direct_dial::state_machine::DialKnownPeer {
            peer: peer.clone(),
            kind: direct_dial::KnownPeerKind::PreviouslyConnected,
            failed_attempts: 0,
            next_attempt: time::Deadline::at(now),
        });
    vec![direct_dial::state_machine::Output::SavePeerBook(
        state.to_peer_book(),
    )]
}
#[cfg(test)]
mod tests {
    use super::handle_peer_listen_addrs;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let _ = state.handle_peer_connected(&identity::PeerId::from("alice"), now);
        let outputs = handle_peer_listen_addrs(
            &mut state,
            &identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            now,
        );
        assert_eq!(outputs.len(), 1);
    }
}
