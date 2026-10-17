use crate::discovery;

pub fn merge(lobby: &mut discovery::Lobby, incoming: discovery::Lobby) {
    let discovery::Lobby(rooms) = incoming;
    for (room, record) in rooms {
        let merged = discovery::freenet::merge_room(lobby.remove(&room), record);
        lobby.insert(room, merged);
    }
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
            },
        );
        discovery::Room {
            room_size: 8,
            members,
        }
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
