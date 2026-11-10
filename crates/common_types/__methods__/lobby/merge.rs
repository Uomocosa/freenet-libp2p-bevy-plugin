use crate::Lobby;
use crate::MAX_LOBBY_ROOMS;
use crate::Room;

pub fn merge(lobby: &mut Lobby, incoming: Lobby) {
    let Lobby(rooms) = incoming;
    for (room, record) in rooms {
        let merged = Room::merge_room(lobby.remove(&room), record);
        lobby.insert(room, merged);
    }
    lobby.drop_rooms_where_every_member_quit();
    lobby.evict_oldest_rooms_until_lobby_fits_capacity(MAX_LOBBY_ROOMS);
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::merge;
    use crate::Lobby;
    use crate::MAX_LOBBY_ROOMS;
    use crate::Peer;
    use crate::PeerId;
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::RoomSize;
    use crate::UnixTime;

    fn record(peer: &str, updated_at: UnixTime) -> Room {
        let mut members = BTreeMap::new();
        members.insert(
            PeerId::from(peer),
            RoomMember {
                peer: Peer {
                    id: PeerId::from(peer),
                    addrs: Vec::new(),
                },
                updated_at,
                display_name: None,
                membership: RoomMembership::Occupant,
            },
        );
        Room {
            room_size: RoomSize::DEFAULT,
            created_at: UnixTime::EPOCH,
            members,
        }
    }

    fn departed_room(updated_at: UnixTime) -> Room {
        let mut record = record("peer", updated_at);
        for room_member in record.members.values_mut() {
            room_member.membership = RoomMembership::Departed;
        }
        record
    }

    #[test]
    fn drops_a_room_when_every_member_is_departed_in_either_merge_order() {
        let room = RoomName::from("room-a");
        let live = record("peer", UnixTime::from_secs(5));
        let departed = departed_room(UnixTime::from_secs(9));
        let pairs = [(live.clone(), departed.clone()), (departed, live)];
        for (base_room, incoming_room) in pairs {
            let mut base = Lobby::from([(room.clone(), base_room)]);
            merge(&mut base, Lobby::from([(room.clone(), incoming_room)]));
            assert!(!base.contains_key(&room));
        }
    }

    #[test]
    fn evicts_the_oldest_room_when_over_capacity() {
        let newest_at = |index: usize| {
            UnixTime::from_secs(u64::try_from(index).unwrap_or_default().saturating_add(1))
        };
        let mut lobby = Lobby::from(
            (0..MAX_LOBBY_ROOMS)
                .map(|index| {
                    (
                        RoomName::from(format!("room-{index:03}")),
                        record("peer", newest_at(index)),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        );
        let incoming = Lobby::from([(
            RoomName::from("z-new"),
            record("peer", newest_at(MAX_LOBBY_ROOMS)),
        )]);
        merge(&mut lobby, incoming);
        assert_eq!(lobby.len(), MAX_LOBBY_ROOMS);
        assert!(!lobby.contains_key(&RoomName::from("room-000")));
        assert!(lobby.contains_key(&RoomName::from("z-new")));
    }

    #[test]
    fn test_usage() {
        let room = RoomName::from("room-a");
        let mut base = Lobby::default();
        base.insert(room.clone(), record("peer", UnixTime::from_secs(5)));
        let mut incoming = Lobby::default();
        incoming.insert(room.clone(), record("peer", UnixTime::from_secs(9)));
        merge(&mut base, incoming);
        let row = base
            .get(&room)
            .and_then(|record| record.members.get(&PeerId::from("peer")));
        assert_eq!(
            row.map(|room_member| room_member.updated_at),
            Some(UnixTime::from_secs(9))
        );
    }
}
