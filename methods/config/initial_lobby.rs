use crate::discovery;
use crate::identity;

#[must_use]
pub fn initial_lobby(
    me: &identity::PeerId,
    room_size: discovery::RoomSize,
    target: Option<&discovery::state_machine::HeartbeatTarget>,
) -> discovery::Lobby {
    let Some(target) = target else {
        return discovery::Lobby::default();
    };
    let room_member = discovery::RoomMember {
        peer: identity::Peer {
            id: me.clone(),
            addrs: target.addrs.clone(),
        },
        updated_at: discovery::UnixTime::now(),
        display_name: Some(target.display_name.clone()),
        membership: target.membership,
    };
    let members = std::collections::BTreeMap::from([(me.clone(), room_member)]);
    discovery::Lobby::from([(
        target.room.clone(),
        discovery::LobbyRoom {
            room_size,
            created_at: target.created_at,
            members,
        },
    )])
}

#[cfg(test)]
mod tests {
    use super::initial_lobby;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let me = identity::PeerId::from("me");
        let empty = initial_lobby(&me, discovery::RoomSize::DEFAULT, None);
        assert_eq!(empty.len(), 0);
        let target = discovery::state_machine::HeartbeatTarget {
            room: identity::RoomName::from("r"),
            addrs: Vec::new(),
            display_name: identity::Username::from("me"),
            membership: discovery::RoomMembership::Occupant,
            created_at: discovery::UnixTime::from(1),
        };
        let seeded = initial_lobby(&me, discovery::RoomSize::DEFAULT, Some(&target));
        assert_eq!(seeded.len(), 1);
    }
}
