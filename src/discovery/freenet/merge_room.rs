use crate::discovery;
use discovery::{Heartbeat, Room};

#[must_use]
pub fn merge_room(current: Option<Room>, incoming: Room) -> Room {
    match current {
        Some(mut existing) => {
            for (peer, heartbeat) in incoming.members {
                let merged = merge_heartbeat(existing.members.remove(&peer), heartbeat);
                existing.members.insert(peer, merged);
            }
            existing.room_size = existing.room_size.max(incoming.room_size);
            existing.created_at = existing.created_at.min(incoming.created_at);
            existing
        }
        None => incoming,
    }
}

// needed helper: last-write-wins on the client-supplied timestamp
fn merge_heartbeat(current: Option<Heartbeat>, incoming: Heartbeat) -> Heartbeat {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::merge_room;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut members = BTreeMap::new();
        members.insert(
            identity::PeerId::from("peer"),
            discovery::Heartbeat {
                addrs: Vec::new(),
                updated_at: discovery::UnixTime::from_secs(5),
                display_name: None,
            },
        );
        let current = discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(200),
            members,
        };
        let mut newer = BTreeMap::new();
        newer.insert(
            identity::PeerId::from("peer"),
            discovery::Heartbeat {
                addrs: Vec::new(),
                updated_at: discovery::UnixTime::from_secs(9),
                display_name: None,
            },
        );
        let incoming = discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(100),
            members: newer,
        };
        let merged = merge_room(Some(current), incoming);
        assert_eq!(merged.created_at, discovery::UnixTime::from_secs(100));
        let row = merged.members.get(&identity::PeerId::from("peer"));
        assert_eq!(
            row.map(|heartbeat| heartbeat.updated_at),
            Some(discovery::UnixTime::from_secs(9))
        );
    }
}
