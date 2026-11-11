use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::discovery;
use discovery::{HOVER_OUTLINE_WIDTH, HoverGlow, ROOM_COLOR, RoomButton, RoomList};
use freenet_libp2p_bevy_core::discovery as core_discovery;

pub fn rebuild_room_list(
    mut commands: Commands,
    rows: Option<Res<core_discovery::LobbyRows>>,
    list: Single<(Entity, Ref<RoomList>)>,
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
        .with_children(|parent| {
            for row in &*shown {
                let label = format!(
                    "> {} [{}/{} online]",
                    row.name.as_str(),
                    row.occupants,
                    row.room_size
                );
                parent.spawn((
                    RoomButton(row.name.clone()),
                    Button,
                    Hovered::default(),
                    HoverGlow::default(),
                    UiTransform::default(),
                    Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![Text::new(label)],
                ));
            }
        });
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::discovery;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::discovery as core_discovery;
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
        app.world_mut().spawn(discovery::RoomList);
        app.add_systems(Update, rebuild_room_list);
        app
    }

    fn app_with_state(state: core_discovery::state_machine::DiscoveryState) -> App {
        let mut app = App::new();
        app.insert_resource(state);
        app.init_resource::<core_discovery::LobbyRows>();
        app.world_mut().spawn(discovery::RoomList);
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

    fn row_labels(app: &mut App) -> Vec<String> {
        let mut rows = app
            .world_mut()
            .query::<(&discovery::RoomButton, &Children)>();
        let mut labels: Vec<String> = rows
            .iter(app.world())
            .map(|(_, children)| {
                children
                    .iter()
                    .filter_map(|child| app.world().get::<Text>(child))
                    .map(|text| text.as_str().to_string())
                    .next()
                    .unwrap_or_default()
            })
            .collect();
        labels.sort();
        labels
    }

    fn button_names(app: &mut App) -> Vec<String> {
        let mut buttons = app.world_mut().query::<&discovery::RoomButton>();
        let mut names: Vec<String> = buttons
            .iter(app.world())
            .map(|button| button.as_str().to_string())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn test_usage() {
        let mut lobby = core_discovery::Lobby::default();
        for name in ["alpha", "beta"] {
            lobby.insert(identity::RoomName::from(name), room(&[]));
        }
        let state = freenet_libp2p_bevy_core::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .build();
        let mut app = app_with_state(state);
        app.update();
        assert_eq!(
            button_names(&mut app),
            vec!["alpha".to_string(), "beta".to_string()]
        );
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
        let state = freenet_libp2p_bevy_core::testing::DiscoveryStateBuilder::new(
            identity::PeerId::from("me"),
        )
        .with_lobby(lobby)
        .build();
        let mut app = app_with_state(state);
        app.update();
        let labels = row_labels(&mut app);
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0], "> alpha [1/8 online]".to_string());
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
        assert_eq!(
            row_labels(&mut app),
            vec!["> alpha [3/8 online]".to_string()]
        );
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
            .query_filtered::<Entity, With<discovery::RoomButton>>();
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

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_state());
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

    fn preview_state() -> core_discovery::state_machine::DiscoveryState {
        let mut lobby = core_discovery::Lobby::default();
        for (name, room_size) in [("alpha", 8), ("beta", 4)] {
            lobby.insert(
                identity::RoomName::from(name),
                core_discovery::Room {
                    room_size: core_discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::new(),
                },
            );
        }
        freenet_libp2p_bevy_core::testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build()
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            discovery::RoomList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn add_room(world: &mut World) {
        let mut state = world.resource_mut::<core_discovery::state_machine::DiscoveryState>();
        state.lobby.insert(
            identity::RoomName::from("gamma"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                created_at: freenet_libp2p_bevy_core::time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::new(),
            },
        );
    }

    #[test]
    fn rebuild_room_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_room_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("two rooms"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("three rooms"),
                    apply: add_room,
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
