use std::time::Duration;

use crate::discovery;

pub fn drop_dead(lobby: &mut discovery::Lobby, now: discovery::UnixTime, ttl: Duration) {
    for record in lobby.values_mut() {
        record.members.retain(|_, room_member| {
            room_member.membership != discovery::RoomMembership::Departed
                && now.elapsed_since(room_member.updated_at) <= ttl
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

    fn room_member(updated_at: discovery::UnixTime) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from("p"),
                addrs: Vec::new(),
            },
            updated_at,
            display_name: None,
            membership: discovery::RoomMembership::Occupant,
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("live"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::from([
                    (
                        identity::PeerId::from("fresh"),
                        room_member(discovery::UnixTime::from_secs(190)),
                    ),
                    (
                        identity::PeerId::from("stale"),
                        room_member(discovery::UnixTime::from_secs(1)),
                    ),
                ]),
            },
        );
        lobby.insert(
            identity::RoomName::from("dead"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::from([(
                    identity::PeerId::from("gone"),
                    room_member(discovery::UnixTime::from_secs(1)),
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
    fn drops_departed_and_expired_holders() {
        let now = discovery::UnixTime::from_secs(100);
        let ttl = Duration::from_secs(90);
        let cases = [
            (discovery::RoomMembership::Occupant, 50, 1),
            (discovery::RoomMembership::HistoryHolder, 50, 1),
            (discovery::RoomMembership::HistoryHolder, 5, 0),
            (discovery::RoomMembership::Departed, 99, 0),
        ];
        for (membership, updated_at, rooms_left) in cases {
            let mut lobby = discovery::Lobby::from([(
                identity::RoomName::from("r"),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: discovery::UnixTime::from_secs(0),
                    members: BTreeMap::from([(
                        identity::PeerId::from("p"),
                        discovery::RoomMember {
                            membership,
                            ..room_member(discovery::UnixTime::from_secs(updated_at))
                        },
                    )]),
                },
            )]);
            drop_dead(&mut lobby, now, ttl);
            assert_eq!(lobby.len(), rooms_left, "{membership:?} at {updated_at}");
        }
    }
}
