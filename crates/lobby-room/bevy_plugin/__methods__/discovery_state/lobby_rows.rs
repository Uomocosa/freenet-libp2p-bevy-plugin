use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;

pub fn lobby_rows(
    state: &DiscoveryState,
    friend_ids: &[identity::PeerId],
) -> Vec<discovery::LobbyRow> {
    let mut rows: Vec<discovery::LobbyRow> = state
        .lobby
        .iter()
        .map(|(name, room)| {
            let occupants = room
                .members
                .values()
                .filter(|member| member.membership == discovery::RoomMembership::Occupant)
                .count();
            let friends = friend_ids
                .iter()
                .filter(|id| {
                    room.members.get(*id).is_some_and(|member| {
                        member.membership == discovery::RoomMembership::Occupant
                    })
                })
                .count();
            let full_at = usize::from(room.room_size.get());
            let occupancy = if occupants == 0 {
                discovery::RoomOccupancy::Empty
            } else if occupants >= full_at {
                discovery::RoomOccupancy::Full
            } else {
                discovery::RoomOccupancy::HasSpace
            };
            discovery::LobbyRow {
                name: name.clone(),
                occupants,
                room_size: room.room_size,
                occupancy,
                friends,
            }
        })
        .collect();
    let join_rank = |occupancy: discovery::RoomOccupancy| match occupancy {
        discovery::RoomOccupancy::HasSpace => 0_u8,
        discovery::RoomOccupancy::Empty => 1,
        discovery::RoomOccupancy::Full => 2,
    };
    rows.sort_by(|a, b| {
        b.friends
            .cmp(&a.friends)
            .then_with(|| join_rank(a.occupancy).cmp(&join_rank(b.occupancy)))
            .then_with(|| b.occupants.cmp(&a.occupants))
            .then_with(|| a.name.cmp(&b.name))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::lobby_rows;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        assert_eq!(lobby_rows(&state, &[]), Vec::new());
    }

    #[test]
    fn occupants_count_only_occupant_members() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    0,
                    testing::room_members(&[
                        ("a", None, discovery::RoomMembership::Occupant),
                        ("b", None, discovery::RoomMembership::Occupant),
                        ("h", None, discovery::RoomMembership::HistoryHolder),
                        ("d", None, discovery::RoomMembership::Departed),
                    ]),
                ),
            )]))
            .build();
        let rows = lobby_rows(&state, &[]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].occupants, 2);
        assert_eq!(
            rows[0].room_size,
            discovery::RoomSize::try_from(8).expect("8")
        );
        assert_eq!(rows[0].occupancy, discovery::RoomOccupancy::HasSpace);
    }

    #[test]
    fn occupancy_kinds() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![
                (
                    "empty",
                    testing::room(discovery::RoomSize::try_from(4).expect("4"), 0, Vec::new()),
                ),
                (
                    "full",
                    testing::room(
                        discovery::RoomSize::try_from(2).expect("2"),
                        0,
                        testing::room_members(&[
                            ("a", None, discovery::RoomMembership::Occupant),
                            ("b", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
                (
                    "spare",
                    testing::room(
                        discovery::RoomSize::try_from(3).expect("3"),
                        0,
                        testing::room_members(&[("a", None, discovery::RoomMembership::Occupant)]),
                    ),
                ),
            ]))
            .build();
        let rows = lobby_rows(&state, &[]);
        let occupancy_of = |name: &str| {
            rows.iter()
                .find(|row| row.name == discovery::RoomName::from(name))
                .expect("row")
                .occupancy
        };
        assert_eq!(occupancy_of("empty"), discovery::RoomOccupancy::Empty);
        assert_eq!(occupancy_of("full"), discovery::RoomOccupancy::Full);
        assert_eq!(occupancy_of("spare"), discovery::RoomOccupancy::HasSpace);
    }

    #[test]
    fn friends_are_counted_per_room() {
        let friends = [identity::PeerId::from("a"), identity::PeerId::from("c")];
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![
                (
                    "alpha",
                    testing::room(
                        discovery::RoomSize::DEFAULT,
                        0,
                        testing::room_members(&[("a", None, discovery::RoomMembership::Occupant)]),
                    ),
                ),
                (
                    "beta",
                    testing::room(
                        discovery::RoomSize::DEFAULT,
                        0,
                        testing::room_members(&[
                            ("c", None, discovery::RoomMembership::Occupant),
                            ("b", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
                (
                    "gamma",
                    testing::room(
                        discovery::RoomSize::DEFAULT,
                        0,
                        testing::room_members(&[(
                            "a",
                            None,
                            discovery::RoomMembership::HistoryHolder,
                        )]),
                    ),
                ),
            ]))
            .build();
        let rows = lobby_rows(&state, &friends);
        let friends_of = |name: &str| {
            rows.iter()
                .find(|row| row.name == discovery::RoomName::from(name))
                .expect("row")
                .friends
        };
        assert_eq!(friends_of("alpha"), 1);
        assert_eq!(friends_of("beta"), 1);
        assert_eq!(friends_of("gamma"), 0);
    }

    #[test]
    fn rooms_with_friends_come_first_then_space_then_empty_then_full() {
        let friends = [
            identity::PeerId::from("a"),
            identity::PeerId::from("b"),
            identity::PeerId::from("c"),
        ];
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![
                (
                    "zeta-empty",
                    testing::room(discovery::RoomSize::try_from(4).expect("4"), 0, Vec::new()),
                ),
                (
                    "full-friend",
                    testing::room(
                        discovery::RoomSize::try_from(2).expect("2"),
                        0,
                        testing::room_members(&[
                            ("a", None, discovery::RoomMembership::Occupant),
                            ("b", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
                (
                    "gamma-full",
                    testing::room(
                        discovery::RoomSize::try_from(2).expect("2"),
                        0,
                        testing::room_members(&[
                            ("x", None, discovery::RoomMembership::Occupant),
                            ("y", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
                (
                    "beta-empty",
                    testing::room(discovery::RoomSize::try_from(4).expect("4"), 0, Vec::new()),
                ),
                (
                    "alpha-friend-full",
                    testing::room(
                        discovery::RoomSize::try_from(2).expect("2"),
                        0,
                        testing::room_members(&[
                            ("a", None, discovery::RoomMembership::Occupant),
                            ("c", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
                (
                    "delta-spare",
                    testing::room(
                        discovery::RoomSize::try_from(4).expect("4"),
                        0,
                        testing::room_members(&[("x", None, discovery::RoomMembership::Occupant)]),
                    ),
                ),
                (
                    "epsilon-spare",
                    testing::room(
                        discovery::RoomSize::try_from(4).expect("4"),
                        0,
                        testing::room_members(&[
                            ("x", None, discovery::RoomMembership::Occupant),
                            ("y", None, discovery::RoomMembership::Occupant),
                        ]),
                    ),
                ),
            ]))
            .build();
        let rows = lobby_rows(&state, &friends);
        let names: Vec<String> = rows
            .iter()
            .map(|row| row.name.as_str().to_string())
            .collect();
        assert_eq!(
            names,
            vec![
                String::from("alpha-friend-full"),
                String::from("full-friend"),
                String::from("epsilon-spare"),
                String::from("delta-spare"),
                String::from("beta-empty"),
                String::from("zeta-empty"),
                String::from("gamma-full"),
            ]
        );
    }
}
