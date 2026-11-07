use crate::direct_dial;
use crate::discovery;
use crate::identity;

pub fn handle_friend_added(
    state: &mut direct_dial::state_machine::DialState,
    peer: identity::Peer,
    now: discovery::UnixTime,
) -> Vec<direct_dial::state_machine::Output> {
    match state
        .entries
        .iter_mut()
        .find(|entry| entry.peer.id == peer.id)
    {
        Some(entry) => {
            entry.kind = direct_dial::KnownPeerKind::Friend;
            if !peer.addrs.is_empty() {
                entry.peer.addrs = peer.addrs;
            }
            entry.failed_attempts = 0;
            entry.next_attempt_at = now;
        }
        None => {
            state
                .entries
                .push(direct_dial::state_machine::DialKnownPeer {
                    peer,
                    kind: direct_dial::KnownPeerKind::Friend,
                    failed_attempts: 0,
                    next_attempt_at: now,
                });
        }
    }
    state.dial_due(now)
}

#[cfg(test)]
mod tests {
    use super::handle_friend_added;
    use crate::direct_dial;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let now = discovery::UnixTime::from(5);
        let mut state = direct_dial::state_machine::DialState::default();
        let outputs = handle_friend_added(
            &mut state,
            identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            now,
        );
        assert_eq!(outputs.len(), 1);
        assert_eq!(state.entries.len(), 1);
    }
}
