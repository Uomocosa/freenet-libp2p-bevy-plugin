use crate::discovery;
use discovery::Lobby;

#[must_use]
pub fn merge_lobby(mut base: Lobby, incoming: Lobby) -> Lobby {
    for (room, record) in incoming {
        let merged = discovery::freenet::merge_room(base.remove(&room), record);
        base.insert(room, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::merge_lobby;
    use crate::discovery;

    fn record(peer: &str, updated_at: discovery::UnixTime) -> discovery::RoomEntry {
        let mut members = BTreeMap::new();
        members.insert(
            net_id::PeerId(peer.to_string()),
            discovery::Presence {
                addrs: Vec::new(),
                updated_at,
            },
        );
        discovery::RoomEntry {
            capacity: 8,
            members,
        }
    }

    #[test]
    fn test_usage() {
        let room = net_id::RoomName::from("room-a");
        let mut base = BTreeMap::new();
        base.insert(
            room.clone(),
            record("peer", discovery::UnixTime::from_secs(5)),
        );
        let mut incoming = BTreeMap::new();
        incoming.insert(
            room.clone(),
            record("peer", discovery::UnixTime::from_secs(9)),
        );
        let merged = merge_lobby(base, incoming);
        let row = merged
            .get(&room)
            .and_then(|record| record.members.get(&net_id::PeerId::from("peer")));
        assert_eq!(
            row.map(|presence| presence.updated_at),
            Some(discovery::UnixTime::from_secs(9))
        );
    }
}
