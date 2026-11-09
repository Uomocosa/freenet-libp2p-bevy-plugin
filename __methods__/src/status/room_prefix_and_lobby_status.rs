use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;
use crate::time;

#[must_use]
pub fn room_prefix_and_lobby_status(
    status: Option<&discovery::Status>,
    game: &identity::GameId,
    room_name: &identity::RoomName,
) -> (
    p2p::HistoryKeyPrefix,
    room_sync::state_machine::RoomLiveness,
) {
    let created_at = status
        .and_then(|status| {
            status
                .lobby
                .get(room_name)
                .map(|record| record.created_at)
                .or_else(|| {
                    status
                        .room
                        .as_ref()
                        .filter(|joined| &joined.name == room_name)
                        .map(|joined| joined.created_at)
                })
        })
        .unwrap_or(time::UnixTime::EPOCH);
    let prefix = p2p::HistoryKeyPrefix::new(game, room_name, created_at);
    let lobby_status = status.map_or(
        room_sync::state_machine::RoomLiveness {
            is_room_still_listed: true,
            has_other_history_holders: false,
        },
        |status| {
            let self_peer_id = status.self_peer_id.as_ref();
            let Some(record) = status.lobby.get(room_name) else {
                let joined = status
                    .room
                    .as_ref()
                    .filter(|joined| &joined.name == room_name);
                let Some(joined) = joined else {
                    return room_sync::state_machine::RoomLiveness {
                        is_room_still_listed: false,
                        has_other_history_holders: false,
                    };
                };
                if joined.how_entered == discovery::HowRoomWasEntered::Created {
                    return room_sync::state_machine::RoomLiveness {
                        is_room_still_listed: true,
                        has_other_history_holders: false,
                    };
                }
                if !status.is_lobby_fetched {
                    return room_sync::state_machine::RoomLiveness {
                        is_room_still_listed: true,
                        has_other_history_holders: true,
                    };
                }
                return room_sync::state_machine::RoomLiveness {
                    is_room_still_listed: false,
                    has_other_history_holders: false,
                };
            };
            let live_members: Vec<(&identity::PeerId, &discovery::RoomMember)> = record
                .members
                .iter()
                .filter(|(_, room_member)| {
                    room_member.membership != discovery::RoomMembership::Departed
                })
                .collect();
            let is_joined_entry = status.room.as_ref().is_some_and(|joined| {
                joined.name == *room_name
                    && joined.how_entered == discovery::HowRoomWasEntered::Joined
            });
            let is_room_still_listed = if is_joined_entry {
                self_peer_id.map_or_else(
                    || !live_members.is_empty(),
                    |self_id| live_members.iter().any(|(peer, _)| *peer != self_id),
                )
            } else {
                !live_members.is_empty()
            };
            let other_live_count = self_peer_id.map_or_else(
                || live_members.len(),
                |self_id| {
                    live_members
                        .iter()
                        .filter(|(peer, _)| *peer != self_id)
                        .count()
                },
            );
            let has_other_history_holder = live_members.iter().any(|(peer, room_member)| {
                let is_other = self_peer_id != Some(*peer);
                is_other && room_member.membership == discovery::RoomMembership::HistoryHolder
            });
            room_sync::state_machine::RoomLiveness {
                is_room_still_listed,
                has_other_history_holders: other_live_count > 0 || has_other_history_holder,
            }
        },
    );
    (prefix, lobby_status)
}
#[cfg(test)]
mod tests {
    use super::room_prefix_and_lobby_status;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let (prefix, lobby_status) = room_prefix_and_lobby_status(None, &game, &room);
        assert_eq!(
            prefix,
            p2p::HistoryKeyPrefix::new(&game, &room, time::UnixTime::EPOCH)
        );
        assert!(lobby_status.is_room_still_listed);
        assert!(!lobby_status.has_other_history_holders);
        let empty = discovery::Status::default();
        let (same_prefix, empty_status) = room_prefix_and_lobby_status(Some(&empty), &game, &room);
        assert_eq!(same_prefix, prefix);
        assert!(!empty_status.is_room_still_listed);
    }

    #[test]
    fn expired_holder_alone_means_the_room_is_gone() {
        use std::collections::BTreeMap;

        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let holder = identity::PeerId::from("holder");
        let me = identity::PeerId::from("me");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::from([(
                    holder.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: holder,
                            addrs: Vec::new(),
                        },
                        updated_at: time::UnixTime::EPOCH,
                        display_name: None,
                        membership: discovery::RoomMembership::Departed,
                    },
                )]),
            },
        );
        let status = discovery::Status {
            lobby,
            self_peer_id: Some(me),
            ..discovery::Status::default()
        };
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&status), &game, &room);
        assert!(!lobby_status.is_room_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn fresh_holder_means_wait_for_history() {
        use std::collections::BTreeMap;

        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let holder = identity::PeerId::from("holder");
        let me = identity::PeerId::from("me");
        let seen_at = time::UnixTime::from_secs(10);
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: seen_at,
                members: BTreeMap::from([(
                    holder.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: holder,
                            addrs: Vec::new(),
                        },
                        updated_at: seen_at,
                        display_name: None,
                        membership: discovery::RoomMembership::HistoryHolder,
                    },
                )]),
            },
        );
        let status = discovery::Status {
            lobby,
            self_peer_id: Some(me),
            ..discovery::Status::default()
        };
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&status), &game, &room);
        assert!(lobby_status.is_room_still_listed);
        assert!(lobby_status.has_other_history_holders);
    }

    #[test]
    fn joiner_with_only_its_own_entry_sees_the_room_as_gone() {
        use std::collections::BTreeMap;

        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let me = identity::PeerId::from("me");
        let gone = identity::PeerId::from("gone");
        let seen_at = time::UnixTime::from_secs(10);
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::from([
                    (
                        me.clone(),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: me.clone(),
                                addrs: Vec::new(),
                            },
                            updated_at: seen_at,
                            display_name: None,
                            membership: discovery::RoomMembership::Occupant,
                        },
                    ),
                    (
                        gone.clone(),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: gone,
                                addrs: Vec::new(),
                            },
                            updated_at: seen_at,
                            display_name: None,
                            membership: discovery::RoomMembership::Departed,
                        },
                    ),
                ]),
            },
        );
        let status = discovery::Status {
            lobby,
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Joined,
            }),
            self_peer_id: Some(me),
            ..discovery::Status::default()
        };
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&status), &game, &room);
        assert!(!lobby_status.is_room_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn creator_with_only_its_own_entry_sees_a_new_room() {
        use std::collections::BTreeMap;

        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let me = identity::PeerId::from("me");
        let seen_at = time::UnixTime::from_secs(10);
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::from([(
                    me.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: me.clone(),
                            addrs: Vec::new(),
                        },
                        updated_at: seen_at,
                        display_name: None,
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        let status = discovery::Status {
            lobby,
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            }),
            self_peer_id: Some(me),
            ..discovery::Status::default()
        };
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&status), &game, &room);
        assert!(lobby_status.is_room_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn late_joiner_with_fetched_empty_lobby_sees_the_room_as_gone() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let me = identity::PeerId::from("me");
        let status = discovery::Status {
            lobby: discovery::Lobby::default(),
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Joining {
                    since: time::UnixTime::EPOCH,
                },
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Joined,
            }),
            self_peer_id: Some(me),
            is_lobby_fetched: true,
            ..discovery::Status::default()
        };
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&status), &game, &room);
        assert!(!lobby_status.is_room_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }
}
