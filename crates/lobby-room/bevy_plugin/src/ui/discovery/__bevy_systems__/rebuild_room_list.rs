use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

pub fn rebuild_room_list(
    mut commands: Commands,
    rows: Option<Res<core_discovery::LobbyRows>>,
    list: Single<(Entity, Ref<ui::discovery::RoomList>)>,
    mut shown: Local<Vec<core_discovery::LobbyRow>>,
) {
    let wanted: Vec<core_discovery::LobbyRow> =
        rows.map(|rows| (**rows).clone()).unwrap_or_default();
    let (list, marker) = list.into_inner();
    if *shown == wanted && !marker.is_added() {
        return;
    }
    *shown = wanted;
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|rooms| {
            for row in &*shown {
                ui::discovery::bevy_systems::spawn_room_row(&mut rooms.reborrow(), row);
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::testing;
    use crate::ui;
    use std::collections::BTreeMap;

    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    use super::rebuild_room_list;

    fn room(members: &[(&str, core_discovery::RoomMembership)]) -> core_discovery::Room {
        core_discovery::Room {
            room_size: core_discovery::RoomSize::DEFAULT,
            created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
            members: members
                .iter()
                .map(|(peer, membership)| {
                    (
                        identity::PeerId::from(*peer),
                        core_discovery::RoomMember {
                            peer: identity::Peer {
                                id: identity::PeerId::from(*peer),
                                addrs: Vec::new(),
                            },
                            display_name: None,
                            updated_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                            membership: *membership,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn app_with(rows: Vec<core_discovery::LobbyRow>) -> App {
        let mut app = App::new();
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.world_mut().spawn(ui::discovery::RoomList);
        app.add_systems(Update, rebuild_room_list);
        app
    }

    fn app_with_state(state: core_discovery::state_machine::DiscoveryState) -> App {
        let mut app = App::new();
        app.insert_resource(state);
        app.init_resource::<core_discovery::LobbyRows>();
        app.world_mut().spawn(ui::discovery::RoomList);
        app.add_systems(
            Update,
            (
                core_discovery::bevy_systems::rebuild_lobby_rows,
                rebuild_room_list,
            )
                .chain(),
        );
        app
    }

    fn list_rows(app: &mut App) -> Vec<Entity> {
        let list = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomList>>()
            .single(app.world())
            .expect("list");
        app.world()
            .entity(list)
            .get::<Children>()
            .expect("list children")
            .iter()
            .collect()
    }

    fn text_of(app: &mut World, entity: Entity) -> String {
        let mut texts = app.query::<&Text>();
        texts.get(app, entity).expect("text").as_str().to_string()
    }

    fn child_with<C: Component>(app: &World, parent: Entity) -> Entity {
        child_marked::<C>(app, parent).expect("marked child")
    }

    fn child_marked<C: Component>(app: &World, parent: Entity) -> Option<Entity> {
        app.entity(parent)
            .get::<Children>()
            .expect("children")
            .iter()
            .find(|child| app.get::<C>(*child).is_some())
    }

    struct RowTexts {
        name: String,
        bubble: String,
        badge: Option<String>,
    }

    fn row_texts(app: &mut App) -> Vec<(String, RowTexts)> {
        list_rows(app)
            .iter()
            .map(|row| {
                let button = app
                    .world()
                    .get::<ui::discovery::RoomButton>(*row)
                    .expect("room button")
                    .as_str()
                    .to_string();
                let clip = child_with::<ui::discovery::RoomNameClip>(app.world_mut(), *row);
                let scroller = child_with::<ui::discovery::RoomNameScroller>(app.world_mut(), clip);
                let name_text = child_with::<Text>(app.world_mut(), scroller);
                let name = text_of(app.world_mut(), name_text);
                let bubble = child_with::<ui::discovery::RoomBubble>(app.world_mut(), *row);
                let bubble_text = child_with::<Text>(app.world_mut(), bubble);
                let bubble_text = text_of(app.world_mut(), bubble_text);
                let badge_entity =
                    child_marked::<ui::discovery::RoomFriendBadge>(app.world_mut(), bubble);
                let badge = badge_entity.map(|badge| {
                    let badge_text = child_with::<Text>(app.world_mut(), badge);
                    text_of(app.world_mut(), badge_text)
                });
                (
                    button,
                    RowTexts {
                        name,
                        bubble: bubble_text,
                        badge,
                    },
                )
            })
            .collect()
    }

    fn bubble_color_of(app: &mut App, button: &str) -> Color {
        let rows = list_rows(app);
        let row = rows
            .iter()
            .copied()
            .find(|row| {
                app.world()
                    .get::<ui::discovery::RoomButton>(*row)
                    .is_some_and(|name| name.as_str() == button)
            })
            .expect("row");
        let bubble = child_with::<ui::discovery::RoomBubble>(app.world_mut(), row);
        let BackgroundColor(color) = app
            .world()
            .get::<BackgroundColor>(bubble)
            .expect("bubble background");
        *color
    }

    #[test]
    fn test_usage() {
        let mut lobby = core_discovery::Lobby::default();
        for name in ["alpha", "beta"] {
            lobby.insert(identity::RoomName::from(name), room(&[]));
        }
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let mut app = app_with_state(state);
        app.update();
        let rows = row_texts(&mut app);
        assert_eq!(
            rows.iter()
                .map(|(button, _)| button.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
        assert!(rows.iter().all(|(_, texts)| texts.badge.is_none()));
    }

    #[test]
    fn the_count_covers_occupant_members_only() {
        let mut lobby = core_discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            room(&[
                ("occupant", core_discovery::RoomMembership::Occupant),
                ("holder", core_discovery::RoomMembership::HistoryHolder),
                ("departed", core_discovery::RoomMembership::Departed),
            ]),
        );
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let mut app = app_with_state(state);
        app.update();
        let rows = row_texts(&mut app);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1.bubble, "1/8".to_string());
    }

    #[test]
    fn rows_come_from_the_lobby_rows_resource() {
        let rows = vec![core_discovery::LobbyRow {
            name: identity::RoomName::from("alpha"),
            occupants: 3,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }];
        let mut app = app_with(rows);
        app.update();
        let rows = row_texts(&mut app);
        assert_eq!(rows[0].1.name, "alpha".to_string());
        assert_eq!(rows[0].1.bubble, "3/8".to_string());
    }

    #[test]
    fn the_bubble_color_follows_the_occupancy() {
        let rows = vec![
            core_discovery::LobbyRow {
                name: identity::RoomName::from("alpha"),
                occupants: 3,
                room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 0,
            },
            core_discovery::LobbyRow {
                name: identity::RoomName::from("beta"),
                occupants: 0,
                room_size: core_discovery::RoomSize::DEFAULT,
                occupancy: core_discovery::RoomOccupancy::Empty,
                friends: 0,
            },
            core_discovery::LobbyRow {
                name: identity::RoomName::from("gamma"),
                occupants: 4,
                room_size: core_discovery::RoomSize::try_from(4).expect("4"),
                occupancy: core_discovery::RoomOccupancy::Full,
                friends: 0,
            },
        ];
        let mut app = app_with(rows);
        app.update();
        assert_eq!(
            bubble_color_of(&mut app, "alpha"),
            ui::discovery::ROOM_BUBBLE_HAS_SPACE_COLOR
        );
        assert_eq!(
            bubble_color_of(&mut app, "beta"),
            ui::discovery::ROOM_BUBBLE_EMPTY_COLOR
        );
        assert_eq!(
            bubble_color_of(&mut app, "gamma"),
            ui::discovery::ROOM_BUBBLE_FULL_COLOR
        );
    }

    #[test]
    fn a_room_with_friends_carries_the_friend_badge() {
        let rows = vec![
            core_discovery::LobbyRow {
                name: identity::RoomName::from("alpha"),
                occupants: 3,
                room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 2,
            },
            core_discovery::LobbyRow {
                name: identity::RoomName::from("beta"),
                occupants: 0,
                room_size: core_discovery::RoomSize::DEFAULT,
                occupancy: core_discovery::RoomOccupancy::Empty,
                friends: 0,
            },
        ];
        let mut app = app_with(rows);
        app.update();
        let rows = row_texts(&mut app);
        assert_eq!(rows[0].1.badge, Some("2".to_string()));
        assert_eq!(rows[1].1.badge, None);
    }

    #[test]
    fn a_long_room_name_is_clipped_unwrapped() {
        let rows = vec![core_discovery::LobbyRow {
            name: identity::RoomName::from("very-long-room-name-that-never-fits"),
            occupants: 1,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }];
        let mut app = app_with(rows);
        app.update();
        let row = list_rows(&mut app)[0];
        let clip = child_with::<ui::discovery::RoomNameClip>(app.world_mut(), row);
        let clip_node = app.world().get::<Node>(clip).expect("clip node");
        assert_eq!(clip_node.overflow.x, bevy::ui::OverflowAxis::Clip);
        let scroller = child_with::<ui::discovery::RoomNameScroller>(app.world_mut(), clip);
        let name_text = child_with::<Text>(app.world_mut(), scroller);
        let layout = app.world().get::<TextLayout>(name_text).expect("layout");
        assert_eq!(layout.linebreak, bevy::text::LineBreak::NoWrap);
    }

    #[test]
    fn fresh_room_row_already_carries_its_hover_styles() {
        let rows = vec![core_discovery::LobbyRow {
            name: identity::RoomName::from("alpha"),
            occupants: 0,
            room_size: core_discovery::RoomSize::DEFAULT,
            occupancy: core_discovery::RoomOccupancy::Empty,
            friends: 0,
        }];
        let mut app = app_with(rows);
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomButton>>();
        let row = rows.single(app.world()).expect("row");
        assert!(
            app.world().get::<UiTransform>(row).is_some(),
            "fresh row carries its hover transform at spawn"
        );
        assert!(
            app.world().get::<Outline>(row).is_some(),
            "fresh row carries its hover outline at spawn"
        );
    }

    fn preview_friends() -> core_direct_dial::state_machine::DialState {
        let mut book = core_direct_dial::PeerBook::default();
        for friend in ["alice", "bob"] {
            assert!(book.add_friend(identity::Peer {
                id: identity::PeerId::from(friend),
                addrs: Vec::new(),
            }));
        }
        core_direct_dial::state_machine::DialState::from_peer_book(
            &book,
            freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
        )
    }

    fn preview_state() -> core_discovery::state_machine::DiscoveryState {
        let mut lobby = core_discovery::Lobby::default();
        let member = |peer: &str| {
            (identity::PeerId::from(peer), {
                core_discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(peer),
                        addrs: Vec::new(),
                    },
                    display_name: None,
                    updated_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                    membership: core_discovery::RoomMembership::Occupant,
                }
            })
        };
        lobby.insert(
            identity::RoomName::from("alpha"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(8)
                    .expect("fixture room size is non-zero"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: ["alice", "bob", "carol"]
                    .into_iter()
                    .map(member)
                    .collect::<BTreeMap<_, _>>(),
            },
        );
        lobby.insert(
            identity::RoomName::from("very-long-room-name-that-never-fits"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: std::iter::once(member("dave")).collect::<BTreeMap<_, _>>(),
            },
        );
        lobby.insert(
            identity::RoomName::from("beta"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(8)
                    .expect("fixture room size is non-zero"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: BTreeMap::new(),
            },
        );
        lobby.insert(
            identity::RoomName::from("gamma"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: ["erin", "frank", "grace", "henry"]
                    .into_iter()
                    .map(member)
                    .collect::<BTreeMap<_, _>>(),
            },
        );
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_state());
        app.insert_resource(preview_friends());
        app.init_resource::<core_discovery::LobbyRows>();
        app.add_systems(Startup, spawn_list);
        app.add_systems(
            Update,
            (
                core_discovery::bevy_systems::rebuild_lobby_rows,
                rebuild_room_list,
            )
                .chain(),
        );
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            ui::discovery::RoomList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn rebuild_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("four rooms sorted"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
