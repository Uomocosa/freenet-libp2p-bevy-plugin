use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;
use std::collections::BTreeSet;

pub fn occupants(state: &DiscoveryState) -> Vec<identity::PeerId> {
    let Some(room) = state.room.as_ref() else {
        return Vec::new();
    };
    let mut present = BTreeSet::new();
    present.insert(state.me.id.clone());
    for (peer, member) in &room.members {
        if member.membership == discovery::RoomMembership::Occupant {
            present.insert(peer.clone());
        }
    }
    present.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::occupants;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        assert_eq!(occupants(&state), Vec::<identity::PeerId>::new());
    }

    #[test]
    fn test_usage_1() {
        let state = testing::ready_state(
            "me",
            "alpha",
            None,
            testing::room_members(&[
                ("occupant", None, discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &[],
        );
        assert_eq!(
            occupants(&state),
            vec![
                identity::PeerId::from("me"),
                identity::PeerId::from("occupant"),
            ]
        );
    }

    #[test]
    fn test_usage_2() {
        let state = testing::ready_state(
            "me",
            "alpha",
            None,
            testing::room_members(&[
                ("me", None, discovery::RoomMembership::Occupant),
                ("other", None, discovery::RoomMembership::Occupant),
            ]),
            &[],
        );
        assert_eq!(
            occupants(&state),
            vec![
                identity::PeerId::from("me"),
                identity::PeerId::from("other"),
            ]
        );
    }
}
