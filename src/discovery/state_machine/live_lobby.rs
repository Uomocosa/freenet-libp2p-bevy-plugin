use std::time::Duration;

use crate::discovery;

#[must_use]
pub fn live_lobby(
    lobby: discovery::Lobby,
    now: discovery::UnixTime,
    ttl: Duration,
) -> discovery::Lobby {
    lobby
        .into_iter()
        .filter_map(|(room, mut record)| {
            record
                .members
                .retain(|_, heartbeat| now.since(heartbeat.updated_at) <= ttl);
            (!record.members.is_empty()).then_some((room, record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::live_lobby;
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
        let mut lobby = discovery::Lobby::new();
        lobby.insert(
            identity::RoomName::from("live"),
            discovery::Room {
                room_size: 8,
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
                room_size: 8,
                members: BTreeMap::from([(
                    identity::PeerId::from("gone"),
                    heartbeat(discovery::UnixTime::from_secs(1)),
                )]),
            },
        );
        let live = live_lobby(
            lobby,
            discovery::UnixTime::from_secs(200),
            discovery::Timing::default().heartbeat_ttl,
        );
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&identity::RoomName::from("live"))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
