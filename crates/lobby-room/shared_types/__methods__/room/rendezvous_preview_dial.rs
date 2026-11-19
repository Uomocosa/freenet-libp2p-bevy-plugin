use crate::Room;
use crate::RoomMember;
use crate::RoomMembership;
use common_types::PeerId;

pub fn rendezvous_preview_dial<'a>(room: &'a Room, me: &PeerId) -> Option<&'a RoomMember> {
    let mut dialable: Vec<&RoomMember> = Vec::new();
    let mut occupants: Vec<&RoomMember> = Vec::new();
    for member in room.members.values() {
        if member.peer.addrs.is_empty() {
            continue;
        }
        if member.membership == RoomMembership::Occupant {
            occupants.push(member);
        } else {
            dialable.push(member);
        }
    }
    if dialable.is_empty() {
        dialable = occupants;
    }
    dialable
        .into_iter()
        .filter(|member| member.peer.id != *me)
        .map(|member| {
            let local = me.as_str();
            let remote = member.peer.id.as_str();
            let combined = PeerId::from(format!("{local}:{remote}"));
            (combined.peer_author(), member)
        })
        .max_by(|(one_score, one), (other_score, other)| {
            one_score
                .cmp(other_score)
                .then_with(|| other.peer.id.cmp(&one.peer.id))
        })
        .map(|(_, pick)| pick)
}

#[cfg(test)]
mod tests {
    use crate::RoomMembership;
    use crate::testing;
    use common_types::PeerAddr;
    use common_types::PeerId;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let room = testing::room_with_history_holders_and_occupants();
        let me = PeerId::from("spectator");
        let pick = room.rendezvous_preview_dial(&me);
        assert!(pick.is_some());
    }

    #[test]
    fn prefers_members_who_are_not_players() {
        let room = testing::room_with_history_holders_and_occupants();
        let me = PeerId::from("spectator");
        let pick = testing::picked_id(room.rendezvous_preview_dial(&me));
        assert!(
            pick == "member-a" || pick == "member-c" || pick == "member-e",
            "picked {pick}, expected a history holder"
        );
    }

    #[test]
    fn two_spectators_spread_over_members() {
        let room = testing::room_with_history_holders_and_occupants();
        let first = PeerId::from("spectator-a");
        let second = PeerId::from("spectator-b");
        let first_pick = testing::picked_id(room.rendezvous_preview_dial(&first));
        let second_pick = testing::picked_id(room.rendezvous_preview_dial(&second));
        assert_ne!(first_pick, second_pick);
    }

    #[test]
    fn the_same_spectator_always_picks_the_same_member() {
        let room = testing::room_with_history_holders_and_occupants();
        let me = PeerId::from("spectator-a");
        let first = testing::picked_id(room.rendezvous_preview_dial(&me));
        let second = testing::picked_id(room.rendezvous_preview_dial(&me));
        assert_eq!(first, second);
    }

    #[test]
    fn never_picks_itself() {
        let room = testing::room_with_history_holders_and_occupants();
        let me = PeerId::from("member-c");
        let pick = testing::picked_id(room.rendezvous_preview_dial(&me));
        assert_ne!(pick, "member-c");
        let solo = testing::room_with_one_member(
            "solo",
            vec![PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            UnixTime::EPOCH,
            RoomMembership::HistoryHolder,
        );
        let lonely = PeerId::from("solo");
        assert!(solo.rendezvous_preview_dial(&lonely).is_none());
    }
}
