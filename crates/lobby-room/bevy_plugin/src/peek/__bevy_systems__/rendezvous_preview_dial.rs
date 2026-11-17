use crate::discovery;
use crate::identity;

#[must_use]
pub(crate) fn rendezvous_preview_dial<'a>(
    me: Option<&identity::PeerId>,
    members: impl IntoIterator<Item = &'a discovery::RoomMember>,
) -> Option<&'a discovery::RoomMember> {
    let mut dialable: Vec<&'a discovery::RoomMember> = Vec::new();
    let mut occupants: Vec<&'a discovery::RoomMember> = Vec::new();
    for member in members {
        if member.peer.addrs.is_empty() {
            continue;
        }
        if member.membership == discovery::RoomMembership::Occupant {
            occupants.push(member);
        } else {
            dialable.push(member);
        }
    }
    if dialable.is_empty() {
        dialable = occupants;
    }
    let Some(me) = me else {
        return dialable.into_iter().next();
    };
    dialable
        .into_iter()
        .filter(|member| member.peer.id != *me)
        .map(|member| {
            let local = me.as_str();
            let remote = member.peer.id.as_str();
            let combined = identity::PeerId::from(format!("{local}:{remote}"));
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
    use crate::discovery;
    use crate::identity;
    use crate::time;

    use super::rendezvous_preview_dial;

    fn member(id: &str, membership: discovery::RoomMembership) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(id),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
            },
            display_name: None,
            updated_at: time::UnixTime::EPOCH,
            membership,
        }
    }

    fn room() -> Vec<discovery::RoomMember> {
        vec![
            member("member-a", discovery::RoomMembership::HistoryHolder),
            member("member-b", discovery::RoomMembership::Occupant),
            member("member-c", discovery::RoomMembership::HistoryHolder),
            member("member-d", discovery::RoomMembership::Occupant),
            member("member-e", discovery::RoomMembership::HistoryHolder),
        ]
    }

    fn picked_id(pick: Option<&discovery::RoomMember>) -> &str {
        pick.expect("a member is picked").peer.id.as_str()
    }

    #[test]
    fn test_usage() {
        let members = room();
        let me = identity::PeerId::from("spectator");
        let pick = rendezvous_preview_dial(Some(&me), members.iter());
        assert!(pick.is_some());
    }

    #[test]
    fn prefers_members_who_are_not_players() {
        let members = room();
        let me = identity::PeerId::from("spectator");
        let pick = picked_id(rendezvous_preview_dial(Some(&me), members.iter()));
        assert!(
            pick == "member-a" || pick == "member-c" || pick == "member-e",
            "picked {pick}, expected a history holder"
        );
    }

    #[test]
    fn two_spectators_spread_over_members() {
        let members = room();
        let first = identity::PeerId::from("spectator-a");
        let second = identity::PeerId::from("spectator-b");
        let first_pick = picked_id(rendezvous_preview_dial(Some(&first), members.iter()));
        let second_pick = picked_id(rendezvous_preview_dial(Some(&second), members.iter()));
        assert_ne!(first_pick, second_pick);
    }

    #[test]
    fn the_same_spectator_always_picks_the_same_member() {
        let members = room();
        let me = identity::PeerId::from("spectator-a");
        let first = picked_id(rendezvous_preview_dial(Some(&me), members.iter()));
        let second = picked_id(rendezvous_preview_dial(Some(&me), members.iter()));
        assert_eq!(first, second);
    }

    #[test]
    fn never_picks_itself() {
        let members = room();
        let me = identity::PeerId::from("member-c");
        let pick = picked_id(rendezvous_preview_dial(Some(&me), members.iter()));
        assert_ne!(pick, "member-c");
        let solo = [member("solo", discovery::RoomMembership::HistoryHolder)];
        let lonely = identity::PeerId::from("solo");
        assert!(rendezvous_preview_dial(Some(&lonely), solo.iter()).is_none());
    }

    #[test]
    fn an_unknown_spectator_falls_back_to_the_first_candidate() {
        let members = room();
        let pick = picked_id(rendezvous_preview_dial(None, members.iter()));
        assert_eq!(pick, "member-a");
    }
}
