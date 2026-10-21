use crate::discovery;

pub fn drop_rooms_where_every_member_quit(lobby: &mut discovery::Lobby) {
    lobby.retain(|_, room| {
        room.members
            .values()
            .any(|room_member| room_member.presence != discovery::Presence::Quit)
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::drop_rooms_where_every_member_quit;
    use crate::{discovery, identity};

    fn room(presence: discovery::Presence) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::DEFAULT,
            created_at: discovery::UnixTime::from_secs(0),
            members: BTreeMap::from([(
                identity::PeerId::from("peer"),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from("peer"),
                        addrs: Vec::new(),
                    },
                    updated_at: discovery::UnixTime::from_secs(1),
                    display_name: None,
                    presence,
                },
            )]),
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = discovery::Lobby::from([
            (
                identity::RoomName::from("live"),
                room(discovery::Presence::InRoom),
            ),
            (
                identity::RoomName::from("holding"),
                room(discovery::Presence::HoldingHistory),
            ),
            (
                identity::RoomName::from("quit"),
                room(discovery::Presence::Quit),
            ),
            (
                identity::RoomName::from("empty"),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: discovery::UnixTime::from_secs(0),
                    members: BTreeMap::new(),
                },
            ),
        ]);
        drop_rooms_where_every_member_quit(&mut lobby);
        let mut rooms: Vec<_> = lobby.keys().map(|room| room.as_str()).collect();
        rooms.sort_unstable();
        assert_eq!(rooms, vec!["holding", "live"]);
    }
}
