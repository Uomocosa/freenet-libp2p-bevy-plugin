use std::time::Duration;

use crate::discovery;

pub fn prune(lobby: &mut discovery::Lobby, now: discovery::UnixTime, ttl: Duration) {
    for record in lobby.values_mut() {
        record
            .members
            .retain(|_, heartbeat| now.since(heartbeat.updated_at) <= ttl);
    }
    lobby.retain(|_, record| !record.members.is_empty());
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::prune;
    use crate::discovery;

    fn heartbeat(updated_at: discovery::UnixTime) -> discovery::Heartbeat {
        discovery::Heartbeat {
            addrs: Vec::new(),
            updated_at,
            display_name: None,
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("live"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                members: BTreeMap::from([
                    (
                        identity::PeerId::from("fresh"),
                        heartbeat(discovery::UnixTime::from_secs(190)),
                    ),
                    (
                        identity::PeerId::from("stale"),
                        heartbeat(discovery::UnixTime::from_secs(1)),
                    ),
                ]),
            },
        );
        lobby.insert(
            identity::RoomName::from("dead"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                members: BTreeMap::from([(
                    identity::PeerId::from("gone"),
                    heartbeat(discovery::UnixTime::from_secs(1)),
                )]),
            },
        );
        prune(
            &mut lobby,
            discovery::UnixTime::from_secs(200),
            discovery::state_machine::HEARTBEAT_TTL,
        );
        let live = lobby;
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&identity::RoomName::from("live"))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
