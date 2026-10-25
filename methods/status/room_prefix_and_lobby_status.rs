use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

#[must_use]
pub fn room_prefix_and_lobby_status(
    status: Option<&discovery::Status>,
    game: &identity::GameId,
    room_name: &identity::RoomName,
) -> (
    p2p::HistoryKeyPrefix,
    room_sync::state_machine::LobbyRoomStatus,
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
        .unwrap_or_else(|| discovery::UnixTime::from(0));
    let prefix = p2p::HistoryKeyPrefix::new(game, room_name, created_at);
    let lobby_status = status.map_or(
        room_sync::state_machine::LobbyRoomStatus {
            is_room_alive_in_lobby: true,
            has_other_lobby_holders: false,
        },
        |status| {
            let self_peer_id = status.self_peer_id.as_ref();
            let ttl = status.lease_ttl;
            let Some(record) = status.lobby.get(room_name) else {
                let is_the_contract_still_publishing_the_room =
                    status.room.as_ref().is_some_and(|joined| {
                        joined.name == *room_name
                            && matches!(joined.phase, discovery::JoinPhase::Joining { .. })
                    });
                return room_sync::state_machine::LobbyRoomStatus {
                    is_room_alive_in_lobby: is_the_contract_still_publishing_the_room,
                    has_other_lobby_holders: false,
                };
            };
            let now = record
                .members
                .values()
                .map(|room_member| room_member.updated_at)
                .max()
                .unwrap_or_else(|| discovery::UnixTime::from(0));
            let is_fresh = |room_member: &discovery::RoomMember| {
                now.elapsed_since(room_member.updated_at) <= ttl
            };
            let live_members: Vec<(&identity::PeerId, &discovery::RoomMember)> = record
                .members
                .iter()
                .filter(|(_, room_member)| {
                    room_member.membership != discovery::RoomMembership::Departed
                        && is_fresh(room_member)
                })
                .collect();
            let live_count = live_members.len();
            let other_live_count = self_peer_id.map_or_else(
                || live_count.saturating_sub(1),
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
            room_sync::state_machine::LobbyRoomStatus {
                is_room_alive_in_lobby: live_count > 0,
                has_other_lobby_holders: other_live_count > 0 || has_other_history_holder,
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

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let (prefix, lobby_status) = room_prefix_and_lobby_status(None, &game, &room);
        assert_eq!(
            prefix,
            p2p::HistoryKeyPrefix::new(&game, &room, discovery::UnixTime::from(0))
        );
        assert!(lobby_status.is_room_alive_in_lobby);
        assert!(!lobby_status.has_other_lobby_holders);
        let empty = discovery::Status::default();
        let (same_prefix, empty_status) = room_prefix_and_lobby_status(Some(&empty), &game, &room);
        assert_eq!(same_prefix, prefix);
        assert!(!empty_status.is_room_alive_in_lobby);
    }
}
