use crate::discovery;
use discovery::{Presence, RoomEntry};

#[must_use]
pub fn merge_room(current: Option<RoomEntry>, incoming: RoomEntry) -> RoomEntry {
    match current {
        Some(mut existing) => {
            for (peer, presence) in incoming.members {
                let merged = merge_presence(existing.members.remove(&peer), presence);
                existing.members.insert(peer, merged);
            }
            existing.capacity = existing.capacity.max(incoming.capacity);
            existing
        }
        None => incoming,
    }
}

// needed helper: last-write-wins on the client-supplied timestamp
fn merge_presence(current: Option<Presence>, incoming: Presence) -> Presence {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::merge_room;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut members = BTreeMap::new();
        members.insert(
            net_id::PeerId::from("peer"),
            discovery::Presence {
                addrs: Vec::new(),
                updated_at: discovery::UnixTime::from_secs(5),
            },
        );
        let current = discovery::RoomEntry {
            capacity: 8,
            members,
        };
        let mut newer = BTreeMap::new();
        newer.insert(
            net_id::PeerId::from("peer"),
            discovery::Presence {
                addrs: Vec::new(),
                updated_at: discovery::UnixTime::from_secs(9),
            },
        );
        let incoming = discovery::RoomEntry {
            capacity: 8,
            members: newer,
        };
        let merged = merge_room(Some(current), incoming);
        let row = merged.members.get(&net_id::PeerId::from("peer"));
        assert_eq!(
            row.map(|presence| presence.updated_at),
            Some(discovery::UnixTime::from_secs(9))
        );
    }
}
