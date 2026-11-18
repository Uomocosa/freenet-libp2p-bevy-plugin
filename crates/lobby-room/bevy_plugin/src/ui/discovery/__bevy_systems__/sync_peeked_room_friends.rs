use crate::direct_dial;
use crate::discovery as core_discovery;
use crate::identity;
use crate::peek;
use crate::ui;
use bevy::prelude::*;

pub fn sync_peeked_room_friends(
    hovered_pill: Res<ui::discovery::HoveredOccupancyPill>,
    preview: Option<Res<peek::RoomPreviewTarget>>,
    state: Option<Res<core_discovery::state_machine::DiscoveryState>>,
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    mut peeked: ResMut<ui::discovery::PeekedRoomFriends>,
) {
    let hovered_pill = hovered_pill.into_inner();
    let friends = match (state, dial, preview) {
        (Some(state), Some(dial), Some(preview)) => (**hovered_pill)
            .clone()
            .filter(|room| preview.zone == **room)
            .map_or_else(Vec::new, |room| {
                let ids: Vec<identity::PeerId> =
                    state.lobby.get(&room).map_or_else(Vec::new, |record| {
                        record
                            .members
                            .keys()
                            .filter(|id| dial.is_friend(id))
                            .cloned()
                            .collect()
                    });
                state.room_friends(&room, &ids)
            }),
        _ => Vec::new(),
    };
    peeked.set_if_neq(ui::discovery::PeekedRoomFriends(friends));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::*;

    use super::sync_peeked_room_friends;
    use crate::direct_dial;
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::peek;
    use crate::testing;
    use crate::time;
    use crate::ui;

    fn room(size: u16, members: &[(&str, Option<&str>)]) -> core_discovery::Room {
        core_discovery::Room {
            room_size: core_discovery::RoomSize::try_from(size).expect("8"),
            created_at: time::UnixTime::EPOCH,
            members: members
                .iter()
                .map(|(peer, display_name)| {
                    (
                        identity::PeerId::from(*peer),
                        core_discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(*peer),
                                addrs: Vec::new(),
                            },
                            display_name: display_name.map(identity::Username::from),
                            updated_at: time::UnixTime::EPOCH,
                            membership: core_discovery::RoomMembership::Occupant,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn state(
        rooms: &[(&str, core_discovery::Room)],
    ) -> core_discovery::state_machine::DiscoveryState {
        let mut lobby = core_discovery::Lobby::default();
        for (name, room) in rooms {
            lobby.insert(core_discovery::RoomName::from(*name), room.clone());
        }
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn dial(friends: &[&str]) -> direct_dial::state_machine::DialState {
        let mut book = direct_dial::PeerBook::default();
        for friend in friends {
            assert!(book.add_friend(identity::Peer {
                id: identity::PeerId::from(*friend),
                addrs: Vec::new(),
            }));
        }
        direct_dial::state_machine::DialState::from_peer_book(&book, time::UnixTime::from_secs(10))
    }

    fn preview(room: &str) -> peek::RoomPreviewTarget {
        peek::RoomPreviewTarget {
            zone: identity::ZoneName::from(room),
            image: Handle::default(),
        }
    }

    fn app_with(
        state: core_discovery::state_machine::DiscoveryState,
        dial: Option<direct_dial::state_machine::DialState>,
        preview: Option<peek::RoomPreviewTarget>,
        pill: Option<&str>,
    ) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(state);
        if let Some(dial) = dial {
            app.insert_resource(dial);
        }
        if let Some(preview) = preview {
            app.insert_resource(preview);
        }
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.init_resource::<ui::discovery::PeekedRoomFriends>();
        **app
            .world_mut()
            .resource_mut::<ui::discovery::HoveredOccupancyPill>() =
            pill.map(core_discovery::RoomName::from);
        app.add_systems(Update, sync_peeked_room_friends);
        app
    }

    fn friends(app: &App) -> Vec<core_discovery::RoomFriend> {
        (**app.world().resource::<ui::discovery::PeekedRoomFriends>()).clone()
    }

    fn friend(peer: &str, name: &str) -> core_discovery::RoomFriend {
        core_discovery::RoomFriend {
            peer: identity::PeerId::from(peer),
            name: identity::Username::from(name),
        }
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(
            state(&[("alpha", room(8, &[("alice", Some("Alice"))]))]),
            Some(dial(&["alice"])),
            Some(preview("alpha")),
            Some("alpha"),
        );
        app.update();
        assert_eq!(friends(&app), vec![friend("alice", "Alice")]);
    }

    #[test]
    fn the_peeked_room_lists_only_its_friends_with_their_ids() {
        let mut app = app_with(
            state(&[(
                "alpha",
                room(8, &[("alice", Some("Alice")), ("bob", Some("Bob"))]),
            )]),
            Some(dial(&["alice"])),
            Some(preview("alpha")),
            Some("alpha"),
        );
        app.update();
        assert_eq!(friends(&app), vec![friend("alice", "Alice")]);
    }

    #[test]
    fn a_pill_for_another_room_publishes_nothing() {
        let mut app = app_with(
            state(&[("alpha", room(8, &[("alice", Some("Alice"))]))]),
            Some(dial(&["alice"])),
            Some(preview("alpha")),
            Some("beta"),
        );
        app.update();
        assert_eq!(friends(&app), Vec::new());
    }

    #[test]
    fn no_preview_target_publishes_nothing() {
        let mut app = app_with(
            state(&[("alpha", room(8, &[("alice", Some("Alice"))]))]),
            Some(dial(&["alice"])),
            None,
            Some("alpha"),
        );
        app.update();
        assert_eq!(friends(&app), Vec::new());
    }

    #[test]
    fn no_dial_state_publishes_nothing() {
        let mut app = app_with(
            state(&[("alpha", room(8, &[("alice", Some("Alice"))]))]),
            None,
            Some(preview("alpha")),
            Some("alpha"),
        );
        app.update();
        assert_eq!(friends(&app), Vec::new());
    }

    #[test]
    fn leaving_the_pill_publishes_nothing() {
        let mut app = app_with(
            state(&[("alpha", room(8, &[("alice", Some("Alice"))]))]),
            Some(dial(&["alice"])),
            Some(preview("alpha")),
            Some("alpha"),
        );
        app.update();
        assert_eq!(friends(&app), vec![friend("alice", "Alice")]);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::HoveredOccupancyPill>() = None;
        app.update();
        assert_eq!(friends(&app), Vec::new());
    }
}
