use crate::Lobby;

pub fn evict_oldest_rooms_until_lobby_fits_capacity(lobby: &mut Lobby, capacity: usize) {
    while lobby.len() > capacity {
        let oldest = lobby
            .iter()
            .min_by_key(|(name, room)| (room.newest_room_member(), name.as_str()))
            .map(|(name, _)| name.clone());
        let Some(oldest) = oldest else {
            break;
        };
        lobby.remove(&oldest);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::evict_oldest_rooms_until_lobby_fits_capacity;
    use crate::Lobby;
    use crate::Peer;
    use crate::PeerId;
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::RoomSize;
    use crate::UnixTime;

    fn room_updated_at(secs: u64) -> Room {
        let room_member = RoomMember {
            peer: Peer {
                id: PeerId::from("peer"),
                addrs: Vec::new(),
            },
            updated_at: UnixTime::from_secs(secs),
            display_name: None,
            membership: RoomMembership::Occupant,
        };
        Room {
            room_size: RoomSize::DEFAULT,
            created_at: UnixTime::EPOCH,
            members: BTreeMap::from([(PeerId::from("peer"), room_member)]),
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = Lobby::from([
            (RoomName::from("old"), room_updated_at(1)),
            (RoomName::from("new"), room_updated_at(9)),
            (RoomName::from("mid"), room_updated_at(5)),
        ]);
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 2);
        assert_eq!(lobby.len(), 2);
        assert!(!lobby.contains_key(&RoomName::from("old")));
        evict_oldest_rooms_until_lobby_fits_capacity(&mut lobby, 5);
        assert_eq!(lobby.len(), 2);
    }
}
