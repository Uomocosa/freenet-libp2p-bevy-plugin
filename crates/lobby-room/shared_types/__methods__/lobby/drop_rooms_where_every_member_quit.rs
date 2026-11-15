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
    use crate::Room;
    use crate::RoomMember;
    use crate::RoomMembership;
    use crate::RoomSize;
    use common_types::Peer;
    use common_types::PeerId;
    use common_types::UnixTime;
    use common_types::ZoneName;

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
            (ZoneName::from("gone"), room_with(RoomMembership::Departed)),
            (ZoneName::from("live"), room_with(RoomMembership::Occupant)),
        ]);
        drop_rooms_where_every_member_quit(&mut lobby);
        assert_eq!(lobby.len(), 1);
        assert!(lobby.contains_key(&ZoneName::from("live")));
    }
}
