use crate::discovery;

pub fn merge(lobby: &mut discovery::Lobby, incoming: discovery::Lobby) {
    let discovery::Lobby(rooms) = incoming;
    for (room, record) in rooms {
        let merged = discovery::freenet::merge_room(lobby.remove(&room), record);
        lobby.insert(room, merged);
    }
    discovery::freenet::drop_rooms_where_every_member_quit(lobby);
    discovery::freenet::evict_oldest_rooms_until_lobby_fits_capacity(
        lobby,
        discovery::freenet::MAX_LOBBY_ROOMS,
    );
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::merge;
    use crate::discovery;

    fn record(peer: &str, updated_at: discovery::UnixTime) -> discovery::Room {
        let mut members = BTreeMap::new();
        members.insert(
            identity::PeerId::from(peer),
            discovery::Heartbeat {
                addrs: Vec::new(),
                updated_at,
                display_name: None,
                presence: discovery::Presence::InRoom,
            },
        );
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(0),
            members,
        }
    }

    fn quit_room(updated_at: discovery::UnixTime) -> discovery::Room {
        let mut record = record("peer", updated_at);
        for heartbeat in record.members.values_mut() {
            heartbeat.presence = discovery::Presence::Quit;
        }
        record
    }

    #[test]
    fn drops_a_room_when_every_member_quit_in_either_direction() {
        let room = identity::RoomName::from("room-a");
        let live = record("peer", discovery::UnixTime::from_secs(5));
        let quit = quit_room(discovery::UnixTime::from_secs(9));
        let pairs = [(live.clone(), quit.clone()), (quit, live)];
        for (base_room, incoming_room) in pairs {
            let mut base = discovery::Lobby::from([(room.clone(), base_room)]);
            merge(
                &mut base,
                discovery::Lobby::from([(room.clone(), incoming_room)]),
            );
            assert!(!base.contains_key(&room));
        }
    }

    #[test]
    fn evicts_the_oldest_room_when_over_capacity() {
        let newest_at = |index: usize| {
            discovery::UnixTime::from_secs(
                u64::try_from(index).unwrap_or_default().saturating_add(1),
            )
        };
        let mut lobby = discovery::Lobby::from(
            (0..discovery::freenet::MAX_LOBBY_ROOMS)
                .map(|index| {
                    (
                        identity::RoomName::from(format!("room-{index:03}")),
                        record("peer", newest_at(index)),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        );
        let incoming = discovery::Lobby::from([(
            identity::RoomName::from("z-new"),
            record("peer", newest_at(discovery::freenet::MAX_LOBBY_ROOMS)),
        )]);
        merge(&mut lobby, incoming);
        assert_eq!(lobby.len(), discovery::freenet::MAX_LOBBY_ROOMS);
        assert!(!lobby.contains_key(&identity::RoomName::from("room-000")));
        assert!(lobby.contains_key(&identity::RoomName::from("z-new")));
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-a");
        let mut base = discovery::Lobby::default();
        base.insert(
            room.clone(),
            record("peer", discovery::UnixTime::from_secs(5)),
        );
        let mut incoming = discovery::Lobby::default();
        incoming.insert(
            room.clone(),
            record("peer", discovery::UnixTime::from_secs(9)),
        );
        merge(&mut base, incoming);
        let row = base
            .get(&room)
            .and_then(|record| record.members.get(&identity::PeerId::from("peer")));
        assert_eq!(
            row.map(|heartbeat| heartbeat.updated_at),
            Some(discovery::UnixTime::from_secs(9))
        );
    }
}
