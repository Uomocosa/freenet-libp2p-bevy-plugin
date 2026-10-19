use std::time::Duration;

use crate::discovery;

pub fn drop_dead(lobby: &mut discovery::Lobby, now: discovery::UnixTime, ttl: Duration) {
    for record in lobby.values_mut() {
        record.members.retain(|_, heartbeat| {
            heartbeat.presence != discovery::Presence::Quit
                && now.elapsed_since(heartbeat.updated_at) <= ttl
        });
    }
    lobby.retain(|_, record| !record.members.is_empty());
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use std::time::Duration;

    use super::drop_dead;
    use crate::discovery;

    fn heartbeat(updated_at: discovery::UnixTime) -> discovery::Heartbeat {
        discovery::Heartbeat {
            addrs: Vec::new(),
            updated_at,
            display_name: None,
            presence: discovery::Presence::InRoom,
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("live"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
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
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::from([(
                    identity::PeerId::from("gone"),
                    heartbeat(discovery::UnixTime::from_secs(1)),
                )]),
            },
        );
        drop_dead(
            &mut lobby,
            discovery::UnixTime::from_secs(200),
            Duration::from_secs(90),
        );
        let live = lobby;
        let rooms: Vec<_> = live.keys().map(|room| room.as_str()).collect();
        assert_eq!(rooms, vec!["live"]);
        let members = live
            .get(&identity::RoomName::from("live"))
            .map_or(0, |record| record.members.len());
        assert_eq!(members, 1);
    }

    #[test]
    fn drops_quit_and_expired_holders() {
        let now = discovery::UnixTime::from_secs(100);
        let ttl = Duration::from_secs(90);
        let cases = [
            (discovery::Presence::InRoom, 50, 1),
            (discovery::Presence::HoldingHistory, 50, 1),
            (discovery::Presence::HoldingHistory, 5, 0),
            (discovery::Presence::Quit, 99, 0),
        ];
        for (presence, updated_at, rooms_left) in cases {
            let mut lobby = discovery::Lobby::from([(
                identity::RoomName::from("r"),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: discovery::UnixTime::from_secs(0),
                    members: BTreeMap::from([(
                        identity::PeerId::from("p"),
                        discovery::Heartbeat {
                            presence,
                            ..heartbeat(discovery::UnixTime::from_secs(updated_at))
                        },
                    )]),
                },
            )]);
            drop_dead(&mut lobby, now, ttl);
            assert_eq!(lobby.len(), rooms_left, "{presence:?} at {updated_at}");
        }
    }
}
