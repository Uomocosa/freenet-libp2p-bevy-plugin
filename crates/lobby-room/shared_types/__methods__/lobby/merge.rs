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
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::testing;
    use common_types::PeerId;
    use common_types::UnixTime;

    #[test]
    fn drops_a_room_when_every_member_is_departed_in_either_merge_order() {
        let room = RoomName::from("room-a");
        let live = testing::room_with_one_member(
            "peer",
            Vec::new(),
            UnixTime::from_secs(5),
            RoomMembership::Occupant,
        );
        let departed = testing::room_with_one_member(
            "peer",
            Vec::new(),
            UnixTime::from_secs(9),
            RoomMembership::Departed,
        );
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
                        testing::room_with_one_member(
                            "peer",
                            Vec::new(),
                            newest_at(index),
                            RoomMembership::Occupant,
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        );
        let incoming = Lobby::from([(
            RoomName::from("z-new"),
            testing::room_with_one_member(
                "peer",
                Vec::new(),
                newest_at(MAX_LOBBY_ROOMS),
                RoomMembership::Occupant,
            ),
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
        base.insert(
            room.clone(),
            testing::room_with_one_member(
                "peer",
                Vec::new(),
                UnixTime::from_secs(5),
                RoomMembership::Occupant,
            ),
        );
        let mut incoming = Lobby::default();
        incoming.insert(
            room.clone(),
            testing::room_with_one_member(
                "peer",
                Vec::new(),
                UnixTime::from_secs(9),
                RoomMembership::Occupant,
            ),
        );
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
