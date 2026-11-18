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
    use std::collections::BTreeMap;

    use super::room_friends;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    const LONG_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";

    fn room(
        size: u16,
        members: &[(&str, Option<&str>, discovery::RoomMembership)],
    ) -> discovery::Room {
        discovery::Room {
            room_size: discovery::RoomSize::try_from(size).expect("fixture room size is non-zero"),
            created_at: time::UnixTime::EPOCH,
            members: members
                .iter()
                .map(|(peer, display_name, membership)| {
                    (
                        identity::PeerId::from(*peer),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(*peer),
                                addrs: Vec::new(),
                            },
                            display_name: display_name.map(identity::Username::from),
                            updated_at: time::UnixTime::EPOCH,
                            membership: *membership,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn state_with(rooms: Vec<(&str, discovery::Room)>) -> DiscoveryState {
        let mut lobby = discovery::Lobby::default();
        for (name, room) in rooms {
            lobby.insert(discovery::RoomName::from(name), room);
        }
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn friend(peer: &str, name: &str) -> discovery::RoomFriend {
        discovery::RoomFriend {
            peer: identity::PeerId::from(peer),
            name: identity::Username::from(name),
        }
    }

    #[test]
    fn test_usage() {
        let friends = room_friends(
            &state_with(Vec::new()),
            &discovery::RoomName::from("alpha"),
            &[],
        );
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn friend_occupants_carry_their_peer_and_name() {
        let state = state_with(vec![(
            "alpha",
            room(
                8,
                &[("alice", Some("Alice"), discovery::RoomMembership::Occupant)],
            ),
        )]);
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, vec![friend("alice", "Alice")]);
    }

    #[test]
    fn a_friend_history_holder_is_not_listed() {
        let state = state_with(vec![(
            "gamma",
            room(
                8,
                &[(
                    "alice",
                    Some("Alice"),
                    discovery::RoomMembership::HistoryHolder,
                )],
            ),
        )]);
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("gamma"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn a_member_without_a_display_name_falls_back_to_the_short_peer_id() {
        let state = state_with(vec![(
            "alpha",
            room(8, &[(LONG_ID, None, discovery::RoomMembership::Occupant)]),
        )]);
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from(LONG_ID)],
        );
        assert_eq!(friends, vec![friend(LONG_ID, "12D3Koo...99RC")]);
    }

    #[test]
    fn an_empty_display_name_falls_back_to_the_short_peer_id() {
        let state = state_with(vec![(
            "alpha",
            room(
                8,
                &[("carol", Some(""), discovery::RoomMembership::Occupant)],
            ),
        )]);
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("carol")],
        );
        assert_eq!(friends, vec![friend("carol", "carol")]);
    }

    #[test]
    fn friends_are_sorted_by_name() {
        let state = state_with(vec![(
            "alpha",
            room(
                8,
                &[
                    ("bob", Some("bob"), discovery::RoomMembership::Occupant),
                    ("alice", Some("Alice"), discovery::RoomMembership::Occupant),
                    ("zoe", Some("Zoe"), discovery::RoomMembership::Occupant),
                ],
            ),
        )]);
        let ids = [
            identity::PeerId::from("bob"),
            identity::PeerId::from("zoe"),
            identity::PeerId::from("alice"),
        ];
        let friends = room_friends(&state, &discovery::RoomName::from("alpha"), &ids);
        assert_eq!(
            friends,
            vec![
                friend("alice", "Alice"),
                friend("zoe", "Zoe"),
                friend("bob", "bob"),
            ]
        );
    }

    #[test]
    fn an_unknown_room_has_no_friends() {
        let state = state_with(Vec::new());
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[identity::PeerId::from("alice")],
        );
        assert_eq!(friends, Vec::new());
    }

    #[test]
    fn an_unknown_peer_is_skipped() {
        let state = state_with(vec![(
            "alpha",
            room(
                8,
                &[("alice", Some("Alice"), discovery::RoomMembership::Occupant)],
            ),
        )]);
        let friends = room_friends(
            &state,
            &discovery::RoomName::from("alpha"),
            &[
                identity::PeerId::from("alice"),
                identity::PeerId::from("missing"),
            ],
        );
        assert_eq!(friends, vec![friend("alice", "Alice")]);
    }
}
