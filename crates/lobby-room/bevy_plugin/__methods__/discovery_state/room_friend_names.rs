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
        .iter()
        .map(|(name, room)| {
            let mut names: Vec<identity::Username> = friend_ids
                .iter()
                .filter(|id| {
                    room.members.get(*id).is_some_and(|member| {
                        member.membership == discovery::RoomMembership::Occupant
                    })
                })
                .map(|id| {
                    room.members
                        .get(id)
                        .and_then(|member| member.display_name.clone())
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| {
                            identity::Username::from(identity::ShortPeerId::from(id).as_str())
                        })
                })
                .collect();
            names.sort_by(|a, b| a.as_str().cmp(b.as_str()));
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

    fn names_of<'a>(
        names: &'a BTreeMap<discovery::RoomName, Vec<identity::Username>>,
        room: &str,
    ) -> &'a [identity::Username] {
        names
            .get(&discovery::RoomName::from(room))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[test]
    fn test_usage() {
        let names = room_friend_names(&state_with(Vec::new()), &[]);
        assert_eq!(names, BTreeMap::new());
    }

    #[test]
    fn names_of_friend_occupants_are_listed() {
        let state = state_with(vec![(
            "alpha",
            room(
                8,
                &[("alice", Some("Alice"), discovery::RoomMembership::Occupant)],
            ),
        )]);
        let names = room_friend_names(&state, &[identity::PeerId::from("alice")]);
        assert_eq!(
            names_of(&names, "alpha"),
            &[identity::Username::from("Alice")]
        );
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
        let names = room_friend_names(&state, &[identity::PeerId::from("alice")]);
        assert_eq!(names_of(&names, "gamma"), []);
    }

    #[test]
    fn a_member_without_a_display_name_falls_back_to_the_short_peer_id() {
        let state = state_with(vec![(
            "alpha",
            room(8, &[(LONG_ID, None, discovery::RoomMembership::Occupant)]),
        )]);
        let names = room_friend_names(&state, &[identity::PeerId::from(LONG_ID)]);
        assert_eq!(
            names_of(&names, "alpha"),
            &[identity::Username::from("12D3Koo...99RC")]
        );
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
        let names = room_friend_names(&state, &[identity::PeerId::from("carol")]);
        assert_eq!(
            names_of(&names, "alpha"),
            &[identity::Username::from("carol")]
        );
    }

    #[test]
    fn names_are_sorted() {
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
        let friends = [
            identity::PeerId::from("bob"),
            identity::PeerId::from("zoe"),
            identity::PeerId::from("alice"),
        ];
        let names = room_friend_names(&state, &friends);
        assert_eq!(
            names_of(&names, "alpha"),
            &[
                identity::Username::from("Alice"),
                identity::Username::from("Zoe"),
                identity::Username::from("bob"),
            ]
        );
    }
}
