use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::time;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn room_prefix_and_lobby_status(
    state: Option<&DiscoveryState>,
    game: &identity::GameId,
    room_name: &discovery::RoomName,
) -> (
    p2p::HistoryKeyPrefix,
    replica_sync::state_machine::ZoneLiveness,
) {
    let created_at = state
        .and_then(|state| {
            state
                .lobby
                .get(room_name)
                .map(|record| record.created_at)
                .or_else(|| {
                    state
                        .room
                        .as_ref()
                        .filter(|joined| &joined.name == room_name)
                        .map(|joined| joined.created_at)
                })
        })
        .unwrap_or(time::UnixTime::EPOCH);
    let prefix = p2p::HistoryKeyPrefix::new(game, room_name, created_at);
    let lobby_status = state.map_or(
        replica_sync::state_machine::ZoneLiveness {
            is_zone_still_listed: true,
            has_other_history_holders: false,
        },
        |state| {
            let self_peer_id = &state.me.id;
            let Some(record) = state.lobby.get(room_name) else {
                let joined = state
                    .room
                    .as_ref()
                    .filter(|joined| &joined.name == room_name);
                let Some(joined) = joined else {
                    return replica_sync::state_machine::ZoneLiveness {
                        is_zone_still_listed: false,
                        has_other_history_holders: false,
                    };
                };
                if joined.how_entered == discovery::HowRoomWasEntered::Created {
                    return replica_sync::state_machine::ZoneLiveness {
                        is_zone_still_listed: true,
                        has_other_history_holders: false,
                    };
                }
                if !state.is_lobby_fetched {
                    return replica_sync::state_machine::ZoneLiveness {
                        is_zone_still_listed: true,
                        has_other_history_holders: true,
                    };
                }
                return replica_sync::state_machine::ZoneLiveness {
                    is_zone_still_listed: false,
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
            let is_joined_entry = state.room.as_ref().is_some_and(|joined| {
                joined.name == *room_name
                    && joined.how_entered == discovery::HowRoomWasEntered::Joined
            });
            let is_zone_still_listed = if is_joined_entry {
                live_members.iter().any(|(peer, _)| *peer != self_peer_id)
            } else {
                !live_members.is_empty()
            };
            let other_live_count = live_members
                .iter()
                .filter(|(peer, _)| *peer != self_peer_id)
                .count();
            let has_other_history_holder = live_members.iter().any(|(peer, room_member)| {
                let is_other = *peer != self_peer_id;
                is_other && room_member.membership == discovery::RoomMembership::HistoryHolder
            });
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed,
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
    use crate::testing;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let (prefix, lobby_status) = room_prefix_and_lobby_status(None, &game, &room);
        assert_eq!(
            prefix,
            p2p::HistoryKeyPrefix::new(&game, &room, time::UnixTime::EPOCH)
        );
        assert!(lobby_status.is_zone_still_listed);
        assert!(!lobby_status.has_other_history_holders);
        let empty = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let (same_prefix, empty_status) = room_prefix_and_lobby_status(Some(&empty), &game, &room);
        assert_eq!(same_prefix, prefix);
        assert!(!empty_status.is_zone_still_listed);
    }

    #[test]
    fn expired_holder_alone_means_the_room_is_gone() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let lobby = testing::lobby_with(vec![(
            "room",
            testing::room(
                discovery::RoomSize::DEFAULT,
                0,
                testing::room_members(&[("holder", None, discovery::RoomMembership::Departed)]),
            ),
        )]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&state), &game, &room);
        assert!(!lobby_status.is_zone_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn fresh_holder_means_wait_for_history() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let lobby = testing::lobby_with(vec![(
            "room",
            testing::room(
                discovery::RoomSize::DEFAULT,
                10,
                testing::room_members(&[(
                    "holder",
                    None,
                    discovery::RoomMembership::HistoryHolder,
                )]),
            ),
        )]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&state), &game, &room);
        assert!(lobby_status.is_zone_still_listed);
        assert!(lobby_status.has_other_history_holders);
    }

    #[test]
    fn joiner_with_only_its_own_entry_sees_the_room_as_gone() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let lobby = testing::lobby_with(vec![(
            "room",
            testing::room(
                discovery::RoomSize::DEFAULT,
                0,
                testing::room_members(&[
                    ("me", None, discovery::RoomMembership::Occupant),
                    ("gone", None, discovery::RoomMembership::Departed),
                ]),
            ),
        )]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .in_room(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Joined,
            })
            .build();
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&state), &game, &room);
        assert!(!lobby_status.is_zone_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn creator_with_only_its_own_entry_sees_a_new_room() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let lobby = testing::lobby_with(vec![(
            "room",
            testing::room(
                discovery::RoomSize::DEFAULT,
                0,
                testing::room_members(&[("me", None, discovery::RoomMembership::Occupant)]),
            ),
        )]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .in_room(testing::joined_room(
                "room",
                0,
                discovery::JoinPhase::Ready,
                Vec::new(),
            ))
            .build();
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&state), &game, &room);
        assert!(lobby_status.is_zone_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }

    #[test]
    fn late_joiner_with_fetched_empty_lobby_sees_the_room_as_gone() {
        let game = identity::GameId::from("game");
        let room = discovery::RoomName::from("room");
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: room.clone(),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Joining {
                    since: time::UnixTime::EPOCH,
                },
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Joined,
            })
            .lobby_fetched()
            .build();
        let (_, lobby_status) = room_prefix_and_lobby_status(Some(&state), &game, &room);
        assert!(!lobby_status.is_zone_still_listed);
        assert!(!lobby_status.has_other_history_holders);
    }
}
