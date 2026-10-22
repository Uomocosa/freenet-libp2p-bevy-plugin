use crate::discovery;
use discovery::state_machine::{DiscoveryState, Output};

pub fn handle_lobby(
    state: &mut DiscoveryState,
    mut lobby: discovery::Lobby,
    now: discovery::UnixTime,
) -> Vec<Output> {
    lobby.drop_dead(now, state.lease_ttl);
    if lobby == state.lobby {
        return Vec::new();
    }
    state.lobby = lobby;
    let mut outputs = state.collect_room_peers_from_lobby();
    state.align_room_created_at();
    outputs.extend(state.wait_for_contract_self_heal(now));
    outputs.push(Output::Notify(discovery::Event::LobbyChanged));
    outputs
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::handle_lobby;
    use crate::discovery;
    use discovery::state_machine::{DiscoveryState, Output};

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
        });
        let mut members = BTreeMap::new();
        for peer in ["me", "a"] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(peer),
                        addrs: vec![identity::PeerAddr::from("/ip4/1")],
                    },
                    updated_at: discovery::UnixTime::from_secs(3),
                    display_name: None,
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("r"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members,
            },
        );
        assert_eq!(
            handle_lobby(
                &mut state,
                lobby.clone(),
                discovery::UnixTime::from_secs(10)
            ),
            vec![
                Output::NetCommand(crate::p2p::NetCommand::AddKadPeer(identity::Peer {
                    id: identity::PeerId::from("a"),
                    addrs: vec![identity::PeerAddr::from("/ip4/1")],
                })),
                Output::Notify(discovery::Event::LobbyChanged),
            ]
        );
        assert_eq!(state.candidates.len(), 1);
        assert_eq!(
            handle_lobby(&mut state, lobby, discovery::UnixTime::from_secs(10)),
            Vec::new()
        );
    }
}
