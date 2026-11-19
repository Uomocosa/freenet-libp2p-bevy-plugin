use common_types::Peer;
use common_types::PeerAddr;
use common_types::PeerId;
use common_types::UnixTime;

use crate::RoomMember;
use crate::RoomMembership;

#[must_use]
pub fn room_member(
    id: &str,
    addrs: Vec<PeerAddr>,
    updated_at: UnixTime,
    membership: RoomMembership,
) -> RoomMember {
    RoomMember {
        peer: Peer {
            id: PeerId::from(id),
            addrs,
        },
        updated_at,
        display_name: None,
        membership,
    }
}

#[cfg(test)]
mod tests {
    use super::room_member;
    use crate::RoomMembership;
    use common_types::PeerAddr;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let member = room_member(
            "peer",
            vec![PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            UnixTime::from_secs(3),
            RoomMembership::HistoryHolder,
        );
        assert_eq!(member.peer.id.as_str(), "peer");
        assert_eq!(member.peer.addrs.len(), 1);
        assert_eq!(member.updated_at, UnixTime::from_secs(3));
        assert_eq!(member.membership, RoomMembership::HistoryHolder);
    }
}
