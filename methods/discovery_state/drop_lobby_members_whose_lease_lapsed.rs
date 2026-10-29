use crate::discovery;
use discovery::state_machine::{DiscoveryState, Output};

pub fn drop_lobby_members_whose_lease_lapsed(
    state: &mut DiscoveryState,
    now: discovery::UnixTime,
) -> Vec<Output> {
    let members_before: usize = state
        .lobby
        .values()
        .map(|record| record.members.len())
        .sum();
    let rooms_before = state.lobby.len();
    state.lobby.drop_members_whose_lease_lapsed(
        &state.observed_lobby_heartbeats,
        now,
        state.lease_limits,
    );
    let members_after: usize = state
        .lobby
        .values()
        .map(|record| record.members.len())
        .sum();
    if state.lobby.len() != rooms_before || members_after != members_before {
        vec![Output::Notify(discovery::Event::LobbyChanged)]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use super::drop_lobby_members_whose_lease_lapsed;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::{DiscoveryState, Output};

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs =
            drop_lobby_members_whose_lease_lapsed(&mut state, discovery::UnixTime::from(1));
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn dropped_holder_does_not_resurrect_when_the_same_raw_lobby_arrives_again() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.lease_limits = discovery::LobbyLeaseLimits {
            lease_ttl: Duration::from_secs(90),
            remote_clock_skew_margin: Duration::from_secs(600),
        };
        let seen_at = discovery::UnixTime::from(100);
        let raw_lobby = || {
            discovery::Lobby::from([(
                identity::RoomName::from("r"),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: discovery::UnixTime::from(0),
                    members: BTreeMap::from([(
                        identity::PeerId::from("holder"),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from("holder"),
                                addrs: Vec::new(),
                            },
                            updated_at: seen_at,
                            display_name: None,
                            membership: discovery::RoomMembership::HistoryHolder,
                        },
                    )]),
                },
            )])
        };
        let _ = state.handle_lobby(raw_lobby(), seen_at);
        let dropped =
            drop_lobby_members_whose_lease_lapsed(&mut state, discovery::UnixTime::from(500));
        assert_eq!(
            dropped,
            vec![Output::Notify(discovery::Event::LobbyChanged)]
        );
        assert!(state.lobby.is_empty());
        let outputs = state.handle_lobby(raw_lobby(), discovery::UnixTime::from(500));
        assert!(state.lobby.is_empty());
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn tick_with_no_new_lobby_drops_a_stalled_holder_and_emits_lobby_changed() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.lease_limits = discovery::LobbyLeaseLimits {
            lease_ttl: Duration::from_secs(90),
            remote_clock_skew_margin: Duration::from_secs(600),
        };
        let room = identity::RoomName::from("r");
        let holder = identity::PeerId::from("holder");
        let remote_updated_at = discovery::UnixTime::from(100);
        state.lobby = discovery::Lobby::from([(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(0),
                members: BTreeMap::from([(
                    holder.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: holder.clone(),
                            addrs: Vec::new(),
                        },
                        updated_at: remote_updated_at,
                        display_name: None,
                        membership: discovery::RoomMembership::HistoryHolder,
                    },
                )]),
            },
        )]);
        state.observed_lobby_heartbeats.insert(
            (room, holder),
            discovery::state_machine::ObservedLobbyHeartbeat {
                remote_updated_at,
                first_seen_locally_at: discovery::UnixTime::from(100),
            },
        );
        let outputs =
            drop_lobby_members_whose_lease_lapsed(&mut state, discovery::UnixTime::from(500));
        assert!(state.lobby.is_empty());
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::LobbyChanged)]
        );
    }
}
