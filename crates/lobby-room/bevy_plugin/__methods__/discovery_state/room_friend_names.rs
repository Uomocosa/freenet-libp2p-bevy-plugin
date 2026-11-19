use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;

pub fn room_friend_names(
    state: &DiscoveryState,
    friend_ids: &[identity::PeerId],
) -> BTreeMap<discovery::RoomName, Vec<identity::Username>> {
    state
        .lobby
        .keys()
        .map(|name| {
            let names = state
                .room_friends(name, friend_ids)
                .into_iter()
                .map(|friend| friend.name)
                .collect();
            (name.clone(), names)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::room_friend_names;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    const LONG_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let names = room_friend_names(&state, &[]);
        assert_eq!(names, BTreeMap::new());
    }

    #[test]
    fn names_of_friend_occupants_are_listed() {
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
        let names = room_friend_names(&state, &[identity::PeerId::from("alice")]);
        assert_eq!(
            testing::friend_names_in_room(&names, "alpha"),
            &[identity::Username::from("Alice")]
        );
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
        let names = room_friend_names(&state, &[identity::PeerId::from("alice")]);
        assert_eq!(testing::friend_names_in_room(&names, "gamma"), []);
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
        let names = room_friend_names(&state, &[identity::PeerId::from(LONG_ID)]);
        assert_eq!(
            testing::friend_names_in_room(&names, "alpha"),
            &[identity::Username::from("12D3Koo...99RC")]
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
        let names = room_friend_names(&state, &[identity::PeerId::from("carol")]);
        assert_eq!(
            testing::friend_names_in_room(&names, "alpha"),
            &[identity::Username::from("carol")]
        );
    }

    #[test]
    fn names_are_sorted() {
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
        let friends = [
            identity::PeerId::from("bob"),
            identity::PeerId::from("zoe"),
            identity::PeerId::from("alice"),
        ];
        let names = room_friend_names(&state, &friends);
        assert_eq!(
            testing::friend_names_in_room(&names, "alpha"),
            &[
                identity::Username::from("Alice"),
                identity::Username::from("Zoe"),
                identity::Username::from("bob"),
            ]
        );
    }
}
