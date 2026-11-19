use crate::discovery;
use crate::identity;
use crate::time;

#[must_use]
pub fn room_member(
    peer: &str,
    display_name: Option<&str>,
    membership: discovery::RoomMembership,
    addrs: &[&str],
) -> discovery::RoomMember {
    discovery::RoomMember {
        peer: identity::Peer {
            id: identity::PeerId::from(peer),
            addrs: addrs
                .iter()
                .map(|addr| identity::PeerAddr::from(*addr))
                .collect(),
        },
        display_name: display_name.map(identity::Username::from),
        updated_at: time::UnixTime::EPOCH,
        membership,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::time;

    use super::room_member;

    #[test]
    fn test_usage() {
        let member = room_member(
            "alice",
            Some("Alice"),
            discovery::RoomMembership::Occupant,
            &["/ip4/127.0.0.1/tcp/4001"],
        );
        assert_eq!(member.peer.id, identity::PeerId::from("alice"));
        assert_eq!(
            member.peer.addrs,
            vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")]
        );
        assert_eq!(member.display_name, Some(identity::Username::from("Alice")));
        assert_eq!(member.updated_at, time::UnixTime::EPOCH);
        assert_eq!(member.membership, discovery::RoomMembership::Occupant);
    }
}
