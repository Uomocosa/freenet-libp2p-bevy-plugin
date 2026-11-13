use crate::direct_dial;
use crate::identity;
use crate::time;

pub fn handle_friend_added(
    state: &mut direct_dial::state_machine::DialState,
    peer: identity::Peer,
    now: time::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    let mut book_changed = false;
    if let Some(entry) = state
        .entries
        .iter_mut()
        .find(|entry| entry.peer.id == peer.id)
    {
        if entry.kind != direct_dial::KnownPeerKind::Friend {
            book_changed = true;
        }
        if !peer.addrs.is_empty() && entry.peer.addrs != peer.addrs {
            book_changed = true;
        }
        entry.kind = direct_dial::KnownPeerKind::Friend;
        if !peer.addrs.is_empty() {
            entry.peer.addrs = peer.addrs;
        }
        entry.failed_attempts = 0;
        entry.next_attempt = time::Deadline::at(now);
    } else {
        state
            .entries
            .push(direct_dial::state_machine::DialKnownPeer {
                peer,
                kind: direct_dial::KnownPeerKind::Friend,
                failed_attempts: 0,
                next_attempt: time::Deadline::at(now),
            });
        book_changed = true;
    }
    let mut outputs = Vec::new();
    if book_changed {
        outputs.push(direct_dial::state_machine::Output::SavePeerBook(
            state.to_peer_book(),
        ));
    }
    outputs.extend(state.dial_due(now));
    outputs
}
#[cfg(test)]
mod tests {
    use super::handle_friend_added;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_friend_added(
            &mut state,
            identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            now,
        );
        assert_eq!(outputs.len(), 2);
        assert_eq!(state.entries.len(), 1);
    }

    #[test]
    fn adding_a_friend_saves_the_book() {
        let now = time::UnixTime::from_secs(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_friend_added(
            &mut state,
            identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            now,
        );
        assert!(
            outputs.iter().any(|output| matches!(
                output,
                direct_dial::state_machine::Output::SavePeerBook(_)
            ))
        );
    }
}
