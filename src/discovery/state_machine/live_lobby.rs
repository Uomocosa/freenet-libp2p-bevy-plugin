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
                .retain(|_, presence| now.since(presence.updated_at) <= ttl);
            (!record.members.is_empty()).then_some((room, record))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::live_lobby;
    use crate::discovery;

    fn presence(updated_at: discovery::UnixTime) -> discovery::Presence {
        discovery::Presence {
            addrs: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::new();
        lobby.insert(
            net_id::RoomName::from("live"),
            discovery::RoomEntry {
                capacity: 8,
                members: BTreeMap::from([
                    (
                        net_id::PeerId::from("fresh"),
                        presence(discovery::UnixTime::from_secs(190)),
                    ),
                    (
                        net_id::PeerId::from("stale"),
                        presence(discovery::UnixTime::from_secs(1)),
                    ),
                ]),
            },
        );
        lobby.insert(
            net_id::RoomName::from("dead"),
            discovery::RoomEntry {
                capacity: 8,
                members: BTreeMap::from([(
                    net_id::PeerId::from("gone"),
                    presence(discovery::UnixTime::from_secs(1)),
                )]),
            },
        );
        let live = live_lobby(
            lobby,
            discovery::UnixTime::from_secs(200),
            discovery::Timing::default().presence_ttl,
        );
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&net_id::RoomName::from("live"))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }
}
