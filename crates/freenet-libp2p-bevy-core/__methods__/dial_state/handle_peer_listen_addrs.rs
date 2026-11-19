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
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let _ = state.handle_peer_connected(&identity::PeerId::from("alice"));
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

    #[test]
    fn connect_then_quit_before_identify_never_saves_an_addressless_peer() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let connected = state.handle_peer_connected(&identity::PeerId::from("bob"));
        let disconnected = state.handle_peer_disconnected(&identity::PeerId::from("bob"), now);
        assert_eq!(testing::saved_peer_books(&connected).len(), 0);
        assert_eq!(testing::saved_peer_books(&disconnected).len(), 0);
        assert_eq!(state.to_peer_book(), direct_dial::PeerBook::default());
    }

    #[test]
    fn connect_then_identify_saves_one_previously_connected_entry_with_addrs() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let _ = state.handle_peer_connected(&identity::PeerId::from("bob"));
        let outputs = state
            .handle_peer_listen_addrs(&testing::peer("bob", &["/ip4/127.0.0.1/tcp/4002"]), now);
        let books = testing::saved_peer_books(&outputs);
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].len(), 1);
        assert_eq!(
            books[0][0].peer,
            testing::peer("bob", &["/ip4/127.0.0.1/tcp/4002"])
        );
        assert_eq!(
            books[0][0].kind,
            direct_dial::KnownPeerKind::PreviouslyConnected
        );
    }

    #[test]
    fn identify_before_connect_is_dropped_then_connect_saves_nothing() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let early = state
            .handle_peer_listen_addrs(&testing::peer("bob", &["/ip4/127.0.0.1/tcp/4002"]), now);
        let connected = state.handle_peer_connected(&identity::PeerId::from("bob"));
        assert_eq!(early, Vec::new());
        assert_eq!(connected, Vec::new());
        assert_eq!(state.entries, Vec::new());
    }
}
