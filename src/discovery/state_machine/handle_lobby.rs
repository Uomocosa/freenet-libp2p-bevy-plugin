use crate::discovery;
use discovery::state_machine::{Output, State};

pub fn handle_lobby(
    state: &mut State,
    mut lobby: discovery::Lobby,
    now: discovery::UnixTime,
) -> Vec<Output> {
    lobby.prune(now, discovery::state_machine::HEARTBEAT_TTL);
    if lobby == state.lobby {
        return Vec::new();
    }
    state.lobby = lobby;
    discovery::state_machine::seed_candidates(state);
    vec![Output::Notify(discovery::Event::LobbyChanged)]
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::handle_lobby;
    use crate::discovery;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::Heartbeat {
                    addrs: vec![identity::PeerAddr::from("/ip4/1")],
                    updated_at: discovery::UnixTime::from_secs(3),
                    display_name: None,
                },
            );
        }
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("r"),
            discovery::Room {
                room_size: 8,
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
