use crate::Lobby;
use crate::RoomMembership;

pub fn drop_rooms_where_every_member_quit(lobby: &mut Lobby) {
    lobby.retain(|_, room| {
        room.members
            .values()
            .any(|room_member| room_member.membership != RoomMembership::Departed)
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::drop_rooms_where_every_member_quit;
    use crate::Lobby;
    use crate::Peer;
    use crate::PeerId;
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomName;
    use crate::RoomSize;
    use crate::UnixTime;

    fn room_with(membership: RoomMembership) -> Room {
        let room_member = RoomMember {
            peer: Peer {
                id: PeerId::from("peer"),
                addrs: Vec::new(),
            },
            updated_at: UnixTime::from_secs(1),
            display_name: None,
            membership,
        };
        Room {
            room_size: RoomSize::DEFAULT,
            created_at: UnixTime::EPOCH,
            members: BTreeMap::from([(PeerId::from("peer"), room_member)]),
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = Lobby::from([
            (RoomName::from("gone"), room_with(RoomMembership::Departed)),
            (RoomName::from("live"), room_with(RoomMembership::Occupant)),
        ]);
        drop_rooms_where_every_member_quit(&mut lobby);
        assert_eq!(lobby.len(), 1);
        assert!(lobby.contains_key(&RoomName::from("live")));
    }
}
