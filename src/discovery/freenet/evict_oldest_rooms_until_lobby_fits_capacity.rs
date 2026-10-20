use crate::discovery;

pub fn evict_oldest_rooms_until_lobby_fits_capacity(lobby: &mut discovery::Lobby, capacity: usize) {
    while lobby.len() > capacity {
        let oldest = lobby
            .iter()
            .min_by_key(|(name, room)| (newest_heartbeat_in(room), name.as_str()))
            .map(|(name, _)| name.clone());
        let Some(oldest) = oldest else {
            break;
        };
        lobby.remove(&oldest);
    }
}

fn newest_heartbeat_in(room: &discovery::Room) -> discovery::UnixTime {
    room.members
        .values()
        .map(|heartbeat| heartbeat.updated_at)
        .max()
        .unwrap_or(discovery::UnixTime::from_secs(0))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::evict_oldest_rooms_until_lobby_fits_capacity;
    use crate::{discovery, identity};

    fn room_at(updated_at: discovery::UnixTime) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(0),
            members: BTreeMap::from([(
                identity::PeerId::from("peer"),
                discovery::Heartbeat {
                    addrs: Vec::new(),
                    updated_at,
                    display_name: None,
                    presence: discovery::Presence::InRoom,
                },
            )]),
        }
    }

    fn lobby_with_rooms(
        count: usize,
        newest_at: impl Fn(usize) -> discovery::UnixTime,
    ) -> discovery::Lobby {
        discovery::Lobby::from(
            (0..count)
                .map(|index| {
                    (
                        identity::RoomName::from(format!("room-{index:03}")),
                        room_at(newest_at(index)),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        )
    }

    #[test]
    fn test_usage() {
        let mut lobby = lobby_with_rooms(3, |index| {
            discovery::UnixTime::from_secs(u64::try_from(index).unwrap_or_default())
        });
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&identity::RoomName::from("room-000")));
        assert!(lobby.contains_key(&identity::RoomName::from("room-002")));
    }

    #[test]
    fn evicts_the_smaller_name_when_newest_heartbeats_tie() {
        let mut lobby = lobby_with_rooms(3, |_| discovery::UnixTime::from_secs(5));
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&identity::RoomName::from("room-000")));
        assert!(lobby.contains_key(&identity::RoomName::from("room-002")));
    }
}
