use crate::discovery;
use discovery::{Room, RoomMember};

#[must_use]
pub fn merge_room(current: Option<Room>, incoming: Room) -> Room {
    match current {
        Some(mut existing) if existing.created_at == incoming.created_at => {
            for (peer, room_member) in incoming.members {
                let merged = merge_room_member(existing.members.remove(&peer), room_member);
                existing.members.insert(peer, merged);
            }
            existing.room_size = existing.room_size.max(incoming.room_size);
            existing
        }
        Some(existing) if existing.created_at > incoming.created_at => existing,
        _ => incoming,
    }
}

// needed helper: last-write-wins on the client-supplied timestamp
fn merge_room_member(current: Option<RoomMember>, incoming: RoomMember) -> RoomMember {
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
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from("peer"),
                    addrs: Vec::new(),
                },
                updated_at: discovery::UnixTime::from_secs(5),
                display_name: None,
                presence: discovery::Presence::InRoom,
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
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from("peer"),
                    addrs: Vec::new(),
                },
                updated_at: discovery::UnixTime::from_secs(9),
                display_name: None,
                presence: discovery::Presence::InRoom,
            },
        );
        let incoming = discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(200),
            members: newer,
        };
        let recreated = discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(300),
            members: BTreeMap::new(),
        };
        let old_generation = merge_room(Some(recreated.clone()), current.clone());
        assert_eq!(old_generation, recreated);
        let merged = merge_room(Some(current), incoming);
        assert_eq!(merged.created_at, discovery::UnixTime::from_secs(200));
        let row = merged.members.get(&identity::PeerId::from("peer"));
        assert_eq!(
            row.map(|heartbeat| heartbeat.updated_at),
            Some(discovery::UnixTime::from_secs(9))
        );
    }
}
