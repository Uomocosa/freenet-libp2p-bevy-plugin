use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::testing;

#[must_use]
pub fn preview_snapshot() -> discovery::state_machine::DiscoveryState {
    let peers: Vec<freenet::FreenetPeer> = (1..=3_u16)
        .map(|port| freenet::FreenetPeer {
            id: freenet::FreenetPeerId::from(format!("peer-{port}").as_str()),
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        })
        .collect();
    let lobby = testing::lobby_with(vec![
        (
            "alpha",
            testing::room(
                testing::room_size(8),
                0,
                vec![
                    testing::room_member(
                        "alice",
                        Some("alice"),
                        discovery::RoomMembership::Occupant,
                        &[],
                    ),
                    testing::room_member(
                        "bob",
                        Some("bob"),
                        discovery::RoomMembership::Occupant,
                        &[],
                    ),
                ],
            ),
        ),
        ("beta", testing::room(testing::room_size(4), 0, Vec::new())),
        (
            "gamma",
            testing::room(
                testing::room_size(4),
                0,
                ["erin", "frank", "grace", "henry"]
                    .into_iter()
                    .map(|member| {
                        testing::room_member(
                            member,
                            Some(member),
                            discovery::RoomMembership::Occupant,
                            &[],
                        )
                    })
                    .collect(),
            ),
        ),
    ]);
    testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
        .with_freenet_peers(peers)
        .lobby_fetched()
        .with_lobby(lobby)
        .build()
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    use super::preview_snapshot;

    #[test]
    fn test_usage() {
        let state = preview_snapshot();
        assert!(state.is_lobby_fetched);
        assert_eq!(state.freenet_peers.len(), 3);
        assert_eq!(state.lobby.len(), 3);
        assert_eq!(
            state
                .lobby
                .get(&discovery::RoomName::from("gamma"))
                .expect("gamma")
                .members
                .len(),
            4
        );
    }
}
