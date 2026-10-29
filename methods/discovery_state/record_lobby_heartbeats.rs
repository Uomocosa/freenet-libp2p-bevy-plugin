use crate::discovery;
use discovery::state_machine::DiscoveryState;

pub fn record_lobby_heartbeats(
    state: &mut DiscoveryState,
    lobby: &discovery::Lobby,
    now: discovery::UnixTime,
) {
    for (room_name, record) in lobby.iter() {
        for (peer_id, room_member) in &record.members {
            let key = (room_name.clone(), peer_id.clone());
            let is_same_heartbeat = state
                .observed_lobby_heartbeats
                .get(&key)
                .is_some_and(|observed| observed.remote_updated_at == room_member.updated_at);
            if !is_same_heartbeat {
                state.observed_lobby_heartbeats.insert(
                    key,
                    discovery::state_machine::ObservedLobbyHeartbeat {
                        remote_updated_at: room_member.updated_at,
                        first_seen_locally_at: now,
                    },
                );
            }
        }
    }
    state
        .observed_lobby_heartbeats
        .retain(|(room_name, peer_id), _| {
            lobby
                .get(room_name)
                .is_some_and(|record| record.members.contains_key(peer_id))
        });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::record_lobby_heartbeats;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    fn lobby_with(peer: &str, updated_at: discovery::UnixTime) -> discovery::Lobby {
        discovery::Lobby::from([(
            identity::RoomName::from("r"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(0),
                members: BTreeMap::from([(
                    identity::PeerId::from(peer),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from(peer),
                            addrs: Vec::new(),
                        },
                        updated_at,
                        display_name: None,
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        )])
    }

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let lobby = lobby_with("p", discovery::UnixTime::from(10));
        record_lobby_heartbeats(&mut state, &lobby, discovery::UnixTime::from(20));
        assert_eq!(state.observed_lobby_heartbeats.len(), 1);
    }
}
