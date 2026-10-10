use crate::discovery;
use discovery::state_machine::{Output, State};

pub fn handle_lobby(
    state: &mut State,
    lobby: discovery::Lobby,
    now: discovery::UnixTime,
) -> Vec<Output> {
    let lobby = discovery::state_machine::live_lobby(lobby, now, state.timing.presence_ttl);
    if lobby == state.lobby {
        return Vec::new();
    }
    state.lobby = lobby;
    discovery::state_machine::seed_candidates(state);
    vec![Output::Notify(discovery::Event::LobbyChanged)]
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::handle_lobby;
    use crate::discovery;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                net_id::PeerId(peer.to_string()),
                discovery::Presence {
                    addrs: vec![net_id::PeerAddr::from("/ip4/1")],
                    updated_at: discovery::UnixTime::from_secs(3),
                },
            );
        }
        let mut lobby = discovery::Lobby::new();
        lobby.insert(
            net_id::RoomName::from("r"),
            discovery::RoomEntry {
                capacity: 8,
                members,
            },
        );
        assert_eq!(
            handle_lobby(
                &mut state,
                lobby.clone(),
                discovery::UnixTime::from_secs(10)
            ),
            vec![Output::Notify(discovery::Event::LobbyChanged)]
        );
        assert_eq!(state.candidates.len(), 1);
        assert_eq!(
            handle_lobby(&mut state, lobby, discovery::UnixTime::from_secs(10)),
            Vec::new()
        );
    }
}
