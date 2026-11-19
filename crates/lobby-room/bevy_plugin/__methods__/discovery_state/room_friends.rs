use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;

pub fn room_friends(
    state: &DiscoveryState,
    room: &discovery::RoomName,
    friend_ids: &[identity::PeerId],
) -> Vec<discovery::RoomFriend> {
    let Some(record) = state.lobby.get(room) else {
        return Vec::new();
    };
    let mut friends: Vec<discovery::RoomFriend> = friend_ids
        .iter()
        .filter_map(|id| {
            let member = record.members.get(id)?;
            (member.membership == discovery::RoomMembership::Occupant).then(|| {
                discovery::RoomFriend {
                    peer: id.clone(),
                    name: member
                        .display_name
                        .clone()
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| {
                            identity::Username::from(identity::ShortPeerId::from(id).as_str())
                        }),
                }
            })
        })
        .collect();
    friends.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
    friends
}

#[cfg(test)]
mod tests {
    use super::room_friends;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    const LONG_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let friends = room_friends(&state, &discovery::RoomName::from("alpha"), &[]);
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn friend_occupants_carry_their_peer_and_name() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[(
                        "alice",
                        Some("Alice"),
                        discovery::RoomMembership::Occupant,
                    )]),
                ),
            )]))
            .build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, vec![testing::room_friend("alice", "Alice")]);
    }

    #[test]
    fn a_friend_history_holder_is_not_listed() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "gamma",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[(
                        "alice",
                        Some("Alice"),
                        discovery::RoomMembership::HistoryHolder,
                    )]),
                ),
            )]))
            .build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("gamma"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn a_member_without_a_display_name_falls_back_to_the_short_peer_id() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[(LONG_ID, None, discovery::RoomMembership::Occupant)]),
                ),
            )]))
            .build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from(LONG_ID)],
        );
        assert_eq!(
            friends,
            vec![testing::room_friend(LONG_ID, "12D3Koo...99RC")]
        );
    }

    #[test]
    fn an_empty_display_name_falls_back_to_the_short_peer_id() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[(
                        "carol",
                        Some(""),
                        discovery::RoomMembership::Occupant,
                    )]),
                ),
            )]))
            .build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("carol")],
        );
        assert_eq!(friends, vec![testing::room_friend("carol", "carol")]);
    }

    #[test]
    fn friends_are_sorted_by_name() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[
                        ("bob", Some("bob"), discovery::RoomMembership::Occupant),
                        ("alice", Some("Alice"), discovery::RoomMembership::Occupant),
                        ("zoe", Some("Zoe"), discovery::RoomMembership::Occupant),
                    ]),
                ),
            )]))
            .build();
        let ids = [
            identity::PeerId::from("bob"),
            identity::PeerId::from("zoe"),
            identity::PeerId::from("alice"),
        ];
        let friends = room_friends(&state, &discovery::RoomName::from("alpha"), &ids);
        assert_eq!(
            friends,
            vec![
                testing::room_friend("alice", "Alice"),
                testing::room_friend("zoe", "Zoe"),
                testing::room_friend("bob", "bob"),
            ]
        );
    }

    #[test]
    fn an_unknown_room_has_no_friends() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn an_unknown_peer_is_skipped() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[(
                        "alice",
                        Some("Alice"),
                        discovery::RoomMembership::Occupant,
                    )]),
                ),
            )]))
            .build();
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[
                identity::PeerId::from("alice"),
                identity::PeerId::from("missing"),
            ],
        );
        assert_eq!(friends, vec![testing::room_friend("alice", "Alice")]);
    }
}
