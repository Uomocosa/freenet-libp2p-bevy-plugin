use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::App;
use bevy::prelude::*;
use freenet_libp2p_bevy_core::replica_sync;

#[derive(Debug, Default, Clone, Copy)]
pub struct LobbyRoomUiPlugin;

impl bevy::prelude::Plugin for LobbyRoomUiPlugin {
    fn build(&self, app: &mut App) {
        use ui::discovery::bevy_systems as systems;
        use ui::discovery::{EscMenuState, JoinError, LeaveDialog, Screen};

        app.add_plugins(ui::roster::UiPlugin);
        app.init_resource::<JoinError>();
        app.init_resource::<EscMenuState>();
        app.init_resource::<ui::discovery::FriendsPressBeat>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.init_resource::<ui::discovery::StreamedRoom>();
        app.init_resource::<ui::discovery::StreamDisplay>();
        app.add_message::<ui::discovery::CloseStream>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<core_discovery::Event>();
        app.add_message::<replica_sync::JoinFailure>();
        app.add_systems(OnEnter(Screen::Connecting), systems::spawn_connecting);
        app.add_systems(OnEnter(Screen::Lobby), systems::spawn_root);
        app.add_systems(
            OnEnter(Screen::Lobby),
            (systems::reset_room_selection, systems::spawn_room_preview),
        );
        app.add_systems(OnEnter(Screen::Lobby), systems::spawn_room_stream);
        app.add_systems(OnEnter(Screen::JoiningRoom), systems::spawn_joining_room);
        app.add_systems(OnEnter(Screen::JoinFailed), systems::spawn_join_failed);
        app.add_systems(OnEnter(Screen::InRoom), systems::spawn_leave_door);
        app.add_systems(OnEnter(LeaveDialog::Open), systems::spawn_leave_prompt);
        app.add_systems(Startup, systems::spawn_esc_menu);
        app.add_systems(
            Update,
            (
                (
                    systems::record_join_error,
                    systems::record_join_failure,
                    systems::pick_screen,
                )
                    .chain(),
                (
                    systems::handle_menu_keys,
                    systems::close_menu_on_press,
                    systems::start_friends_beat,
                    systems::open_leave_prompt_from_menu,
                    systems::finish_friends_beat,
                )
                    .chain(),
                systems::animate_hover,
                systems::animate_create_press
                    .after(systems::animate_hover)
                    .after(systems::gate_create),
                systems::start_create_shake,
                systems::show_text_cursor_on_room_name_input.run_if(in_state(Screen::Lobby)),
                systems::animate_esc_menu,
                systems::refresh_menu_leave_button,
                systems::spin_spinners.run_if(|screen: Option<Res<State<Screen>>>| {
                    screen.is_some_and(|screen| {
                        matches!(*screen.get(), Screen::Connecting | Screen::Lobby)
                    })
                }),
                systems::show_room_search_tooltip.run_if(in_state(Screen::Lobby)),
                systems::dismiss_join_failed.run_if(in_state(Screen::JoinFailed)),
                (
                    (systems::rebuild_room_list, systems::scroll_room_names).chain(),
                    systems::gate_create,
                    systems::handle_create,
                    (
                        systems::handle_join,
                        systems::close_stream_on_press,
                        systems::toggle_stream_fullscreen_on_press,
                        systems::drop_stale_room_selection,
                        systems::drop_stale_streamed_room,
                        systems::track_room_preview,
                        systems::start_stream_on_click,
                        systems::watch_selected_room,
                        systems::enter_on_double_click,
                        systems::clear_stream_on_room_entry,
                        systems::clear_stream_on_close_request,
                        systems::highlight_selected_room.after(systems::animate_hover),
                        systems::refresh_room_preview,
                        systems::refresh_room_stream,
                        systems::rebuild_room_stream_action,
                        systems::layout_stream_panel,
                        systems::place_room_preview,
                        systems::fit_stream_image,
                    )
                        .chain(),
                )
                    .run_if(in_state(Screen::Lobby)),
                (
                    systems::request_leave,
                    systems::handle_leave_answer.run_if(in_state(LeaveDialog::Open)),
                    systems::animate_door,
                )
                    .chain()
                    .run_if(in_state(Screen::InRoom)),
            ),
        );
    }
    fn finish(&self, app: &mut App) {
        app.init_state::<ui::discovery::Screen>();
        app.add_sub_state::<ui::discovery::LeaveDialog>();
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::freenet;
    use crate::peek;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::ui;
    use std::time::Duration;

    use bevy::input::ButtonState;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::KeyCode;
    use bevy::input::keyboard::KeyboardInput;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::Pressed;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::p2p;
    use freenet_libp2p_bevy_core::plugin;

    use core_discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_core::time;

    #[derive(Default, Resource)]
    struct PreviewState;

    impl replica_sync::traits::ReplicatedState for PreviewState {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &Self::Op) {}

        fn snapshot(&self) -> Self::Snapshot {
            0
        }

        fn merge(&mut self, _snapshot: Self::Snapshot) {}

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            0
        }
    }

    fn test_pattern_image() -> Image {
        let palette: [[u8; 4]; 6] = [
            [38, 70, 96, 255],
            [52, 92, 122, 255],
            [66, 114, 148, 255],
            [82, 138, 176, 255],
            [38, 70, 96, 255],
            [24, 48, 68, 255],
        ];
        let row: Vec<u8> = palette.concat().into_iter().cycle().take(640).collect();
        let data: Vec<u8> = row.iter().copied().cycle().take(64_000).collect();
        Image::new(
            bevy::render::render_resource::Extent3d {
                width: 160,
                height: 100,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        )
    }

    fn clear_preview_targets(world: &mut World) {
        let mut query = world.query_filtered::<Entity, With<spectator::Spectated<PreviewState>>>();
        let entities: Vec<Entity> = query.iter(world).collect();
        for entity in entities {
            world.despawn(entity);
        }
        world.remove_resource::<peek::RoomPreviewTarget>();
        world.remove_resource::<peek::RoomStreamTarget>();
        **world.resource_mut::<ui::discovery::SelectedRoom>() = None;
        **world.resource_mut::<ui::discovery::StreamedRoom>() = None;
        *world.resource_mut::<ui::discovery::StreamDisplay>() = ui::discovery::StreamDisplay::Panel;
    }

    fn spectate_zone(
        world: &mut World,
        slot: spectator::SpectateSlot,
        room: &str,
        freshness: spectator::SpectateFreshness,
    ) {
        let image = world
            .resource_mut::<Assets<Image>>()
            .add(test_pattern_image());
        let zone = identity::ZoneName::from(room);
        match slot {
            spectator::SpectateSlot::Preview => {
                world.insert_resource(peek::RoomPreviewTarget {
                    zone: zone.clone(),
                    image,
                });
            }
            spectator::SpectateSlot::Stream => {
                world.insert_resource(peek::RoomStreamTarget {
                    zone: zone.clone(),
                    image,
                });
            }
        }
        world.spawn(spectator::Spectated::<PreviewState> {
            slot,
            zone,
            state: PreviewState,
            freshness,
        });
    }

    fn show_stream(world: &mut World, room: &str) {
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from(room));
        spectate_zone(
            world,
            spectator::SpectateSlot::Stream,
            room,
            spectator::SpectateFreshness::Live,
        );
    }

    fn preview_build_with(app: &mut App, rows: core_discovery::LobbyRows) {
        app.init_resource::<Assets<Image>>();
        app.add_message::<core_discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.insert_resource(rows);
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.add_systems(
            Update,
            pin_hover
                .before(ui::discovery::bevy_systems::track_room_preview)
                .before(ui::discovery::bevy_systems::animate_hover),
        );
        app.add_systems(
            Update,
            pin_rooms_header_hover.before(ui::discovery::bevy_systems::show_room_search_tooltip),
        );
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::show_preview_image::<PreviewState>,
                ui::discovery::bevy_systems::show_stream_image::<PreviewState>,
            ),
        );
    }

    fn preview_build(app: &mut App) {
        preview_build_with(app, preview_rows());
    }

    fn long_preview_build(app: &mut App) {
        preview_build_with(app, preview_rows_long());
    }

    fn preview_rows() -> core_discovery::LobbyRows {
        core_discovery::LobbyRows(vec![
            core_discovery::LobbyRow {
                name: core_discovery::RoomName::from("alpha"),
                occupants: 2,
                room_size: core_discovery::RoomSize::try_from(8)
                    .expect("fixture room size is non-zero"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 1,
            },
            core_discovery::LobbyRow {
                name: core_discovery::RoomName::from("beta"),
                occupants: 0,
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                occupancy: core_discovery::RoomOccupancy::Empty,
                friends: 0,
            },
            core_discovery::LobbyRow {
                name: core_discovery::RoomName::from("gamma"),
                occupants: 4,
                room_size: core_discovery::RoomSize::try_from(4)
                    .expect("fixture room size is non-zero"),
                occupancy: core_discovery::RoomOccupancy::Full,
                friends: 0,
            },
        ])
    }

    fn preview_rows_long() -> core_discovery::LobbyRows {
        let mut rows = vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 1,
            room_size: core_discovery::RoomSize::try_from(16)
                .expect("fixture room size is non-zero"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }];
        for index in 2..=12_u8 {
            rows.push(core_discovery::LobbyRow {
                name: core_discovery::RoomName::from(format!("room-{index:02}").as_str()),
                occupants: usize::from(index),
                room_size: core_discovery::RoomSize::try_from(16)
                    .expect("fixture room size is non-zero"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 0,
            });
        }
        core_discovery::LobbyRows(rows)
    }

    fn preview_snapshot() -> DiscoveryState {
        let peers: Vec<freenet::FreenetPeer> = (1..=3_u16)
            .map(|port| freenet::FreenetPeer {
                id: freenet::FreenetPeerId::from(format!("peer-{port}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            })
            .collect();
        let mut lobby = core_discovery::Lobby::default();
        let rooms: [(&str, u16, &[&str]); 3] = [
            ("alpha", 8, &["alice", "bob"]),
            ("beta", 4, &[]),
            ("gamma", 4, &["erin", "frank", "grace", "henry"]),
        ];
        for (name, room_size, member_names) in rooms {
            lobby.insert(
                core_discovery::RoomName::from(name),
                core_discovery::Room {
                    room_size: core_discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: time::UnixTime::EPOCH,
                    members: member_names
                        .iter()
                        .map(|member| {
                            (
                                identity::PeerId::from(*member),
                                core_discovery::RoomMember {
                                    peer: identity::Peer {
                                        id: identity::PeerId::from(*member),
                                        addrs: Vec::new(),
                                    },
                                    display_name: Some(identity::Username::from(*member)),
                                    updated_at: time::UnixTime::EPOCH,
                                    membership: core_discovery::RoomMembership::Occupant,
                                },
                            )
                        })
                        .collect(),
                },
            );
        }
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_freenet_peers(peers)
            .lobby_fetched()
            .with_lobby(lobby)
            .build()
    }

    const fn preview_noop(_world: &mut World) {}

    #[derive(Component)]
    struct PinHover;

    fn pin_hover(mut commands: Commands, pinned: Query<Entity, With<PinHover>>) {
        for entity in &pinned {
            commands.entity(entity).insert(Hovered(true));
        }
    }

    fn hover_all(world: &mut World) {
        let mut inputs = world
            .query_filtered::<&mut bevy::text::EditableText, With<ui::discovery::RoomNameInput>>();
        if let Ok(mut input) = inputs.single_mut(world) {
            input.queue_edit(bevy::text::TextEdit::Insert("gamma".into()));
        }
        let mut glows = world.query_filtered::<Entity, With<ui::discovery::HoverGlow>>();
        let hovered: Vec<Entity> = glows.iter(world).collect();
        for entity in hovered {
            world.entity_mut(entity).insert((
                Hovered(true),
                PinHover,
                ui::discovery::HoverGlow(1.0),
            ));
        }
    }

    #[derive(Component)]
    struct PinRoomsHeaderHover;

    fn pin_rooms_header_hover(
        mut commands: Commands,
        pinned: Query<Entity, With<PinRoomsHeaderHover>>,
    ) {
        for entity in &pinned {
            commands.entity(entity).insert(Hovered(true));
        }
    }

    fn hover_rooms_header(world: &mut World) {
        let mut indicators =
            world.query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
        let indicators: Vec<Entity> = indicators.iter(world).collect();
        for entity in indicators {
            world
                .entity_mut(entity)
                .insert((Hovered(true), PinRoomsHeaderHover));
        }
    }

    fn hover_row(world: &mut World, room: &str) {
        let mut rows = world.query::<(Entity, &ui::discovery::RoomButton)>();
        let targets: Vec<Entity> = rows
            .iter(world)
            .filter(|(_entity, button)| button.as_str() == room)
            .map(|(entity, _button)| entity)
            .collect();
        for entity in targets {
            world.entity_mut(entity).insert((Hovered(true), PinHover));
        }
    }

    fn select_room_value(world: &mut World, room: &str) {
        **world.resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from(room));
        hover_row(world, room);
        spectate_zone(
            world,
            spectator::SpectateSlot::Preview,
            room,
            spectator::SpectateFreshness::Live,
        );
    }

    fn select_room(world: &mut World) {
        select_room_value(world, "alpha");
    }

    fn select_full_room(world: &mut World) {
        select_room_value(world, "gamma");
    }

    fn stream_only(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "alpha");
    }

    fn stream_and_preview(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "alpha");
        select_room_value(world, "beta");
    }

    fn stream_fullscreen(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "alpha");
        *world.resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
    }

    fn stream_room_full(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "gamma");
    }

    fn stream_placeholder(world: &mut World) {
        clear_preview_targets(world);
        **world.resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        world.spawn(spectator::Spectated::<PreviewState> {
            slot: spectator::SpectateSlot::Stream,
            zone: identity::ZoneName::from("alpha"),
            state: PreviewState,
            freshness: spectator::SpectateFreshness::Bootstrapping,
        });
    }

    fn long_top_row(world: &mut World) {
        clear_preview_targets(world);
        select_room_value(world, "alpha");
    }

    fn long_bottom_row(world: &mut World) {
        clear_preview_targets(world);
        select_room_value(world, "room-10");
    }

    fn long_top_row_with_stream(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "room-09");
        select_room_value(world, "alpha");
    }

    fn long_bottom_row_with_stream(world: &mut World) {
        clear_preview_targets(world);
        show_stream(world, "alpha");
        select_room_value(world, "room-10");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..3 {
            app.update();
        }
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::CreateRoomButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }

    fn count<C: Component>(app: &mut App) -> usize {
        let mut query = app.world_mut().query_filtered::<(), With<C>>();
        query.iter(app.world()).count()
    }

    fn preview_visibility(app: &mut App) -> Visibility {
        let mut panels = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::discovery::RoomPreviewPanel>>();
        *panels.single(app.world()).expect("preview panel")
    }

    #[test]
    fn the_lobby_spawns_one_preview_and_one_stream_panel() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.insert_resource(preview_rows());
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.add_systems(
            Update,
            pin_hover
                .before(ui::discovery::bevy_systems::track_room_preview)
                .before(ui::discovery::bevy_systems::animate_hover),
        );
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(count::<ui::discovery::RoomPreviewPanel>(&mut app), 1);
        assert_eq!(count::<ui::discovery::RoomStreamPanel>(&mut app), 1);
        assert_eq!(preview_visibility(&mut app), Visibility::Hidden);
        let mut rows = app
            .world_mut()
            .query::<(Entity, &ui::discovery::RoomButton)>();
        let row = rows
            .iter(app.world())
            .find(|(_entity, button)| button.as_str() == "alpha")
            .map(|(entity, _button)| entity)
            .expect("the alpha room row");
        app.world_mut()
            .entity_mut(row)
            .insert((Hovered(true), PinHover));
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            Some(core_discovery::RoomName::from("alpha")),
            "the pinned hover keeps the selection"
        );
        assert_eq!(
            preview_visibility(&mut app),
            Visibility::Visible,
            "the hovered room shows its preview card"
        );
    }

    #[test]
    fn default_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("default_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby with three rooms"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("all hovered"),
                    apply: hover_all,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("rooms header hovered"),
                    apply: hover_rooms_header,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room hovered"),
                    apply: select_room,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("full room selected"),
                    apply: select_full_room,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream only"),
                    apply: stream_only,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream and preview of beta"),
                    apply: stream_and_preview,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream fullscreen"),
                    apply: stream_fullscreen,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream room full"),
                    apply: stream_room_full,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream placeholder"),
                    apply: stream_placeholder,
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

    #[test]
    fn long_room_list_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("long_room_list"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: long_preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("preview at the top row"),
                    apply: long_top_row,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("preview at the last row"),
                    apply: long_bottom_row,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream with preview at the top row"),
                    apply: long_top_row_with_stream,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream with preview at the last row"),
                    apply: long_bottom_row_with_stream,
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

    fn in_room_fixture() -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("ui-flow"))
            .in_room(core_discovery::JoinedRoom {
                name: core_discovery::RoomName::from("ui-room"),
                created_at: time::UnixTime::EPOCH,
                phase: core_discovery::JoinPhase::Ready,
                members: core_discovery::Members::new(),
                how_entered: core_discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn default_ui_app(network: &testing::NetworkArgs) -> App {
        let game = identity::GameId::from("ui-flow");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(InputPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.insert_resource(core_discovery::FreenetPort(0));
        app.add_plugins(testing::FakeNetworkPlugin::<u32>::from(network));
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u32>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(core_direct_dial::FriendsPlugin::new(
            core_direct_dial::Config::from(None),
        ));
        app.add_plugins(core_discovery::Plugin::new(core_discovery::Config {
            display_name: identity::Username::from("tester"),
            game,
            ..core_discovery::Config::default()
        }));
        app.world_mut().insert_resource(in_room_fixture());
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.add_plugins(ui::direct_dial::FriendsUiPlugin);
        app.finish();
        app
    }

    fn keyboard(key: KeyCode, state: ButtonState) -> KeyboardInput {
        KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    fn press(app: &mut App, key: KeyCode) {
        app.world_mut()
            .write_message(keyboard(key, ButtonState::Pressed));
        app.update();
        app.world_mut()
            .write_message(keyboard(key, ButtonState::Released));
        app.update();
    }

    fn wait_for(app: &mut App, mut is_ready: impl FnMut(&mut App) -> bool) {
        for _ in 0..400 {
            app.update();
            if is_ready(app) {
                return;
            }
        }
        panic!("the waited-for state never arrived");
    }

    fn esc_menu_visibility(app: &mut App) -> Visibility {
        let mut menus = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::discovery::EscMenu>>();
        *menus.single(app.world()).expect("esc menu root")
    }

    fn drawer_visibility(app: &mut App) -> Visibility {
        let mut roots = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::direct_dial::FriendsDrawer>>();
        *roots.single(app.world()).expect("drawer root")
    }

    fn drawer_slide(app: &mut App) -> f32 {
        let mut panels = app
            .world_mut()
            .query_filtered::<&ui::direct_dial::FriendsDrawerSlide, ()>();
        **panels.single(app.world()).expect("drawer panel")
    }

    fn friends_button_pressed(app: &mut App) -> bool {
        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::EscMenuFriendsButton>>();
        let button = buttons.single(app.world()).expect("friends button");
        app.world().get::<Pressed>(button).is_some()
    }

    fn copy_toast_text(app: &mut App) -> String {
        let mut toasts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::direct_dial::CopyToast>>();
        toasts
            .single(app.world())
            .expect("copy toast")
            .as_str()
            .to_string()
    }

    fn chip_label(app: &mut App) -> String {
        let mut labels = app
            .world_mut()
            .query_filtered::<&Text, With<ui::direct_dial::OwnAddressText>>();
        labels
            .single(app.world())
            .expect("address chip")
            .as_str()
            .to_string()
    }

    fn is_chip_waiting(app: &mut App) -> bool {
        chip_label(app).contains("waiting")
    }

    fn is_menu_visible(app: &mut App) -> bool {
        esc_menu_visibility(app) == Visibility::Visible
    }

    fn is_drawer_fully_open(app: &mut App) -> bool {
        (drawer_slide(app) - ui::constants::DRAWER_SLIDE_SECS).abs() < f32::EPSILON
    }

    fn is_drawer_visible(app: &mut App) -> bool {
        drawer_visibility(app) == Visibility::Visible
    }

    fn is_drawer_hidden(app: &mut App) -> bool {
        drawer_visibility(app) == Visibility::Hidden
    }

    fn is_leave_prompt_open(app: &mut App) -> bool {
        let mut prompts = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::LeavePrompt>>();
        prompts.iter(app.world()).count() == 1
    }

    fn is_leave_prompt_closed(app: &mut App) -> bool {
        let mut prompts = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::LeavePrompt>>();
        prompts.iter(app.world()).count() == 0
    }

    fn click_friends_in_menu(app: &mut App) {
        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::EscMenuFriendsButton>>();
        let button = buttons.single(app.world()).expect("friends button");
        app.world_mut().entity_mut(button).insert(Pressed);
    }

    fn click_leave_in_menu(app: &mut App) {
        let mut buttons = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::EscMenuLeaveButton>>();
        let button = buttons.single(app.world()).expect("leave button");
        app.world_mut().entity_mut(button).insert(Pressed);
    }

    fn click_copy_chip(app: &mut App) {
        let mut chips = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::OwnAddressChip>>();
        let chip = chips.single(app.world()).expect("address chip");
        app.world_mut().entity_mut(chip).insert(Pressed);
    }

    fn esc_opens_and_closes_the_menu(app: &mut App) {
        press(app, KeyCode::Escape);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Visible,
            "Esc opens the menu"
        );
        press(app, KeyCode::Escape);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Hidden,
            "Esc closes the menu"
        );
    }

    fn the_copy_chip_shows_its_toast(app: &mut App) {
        press(app, KeyCode::Tab);
        wait_for(app, is_drawer_fully_open);
        assert_eq!(
            drawer_visibility(app),
            Visibility::Visible,
            "Tab opens the drawer"
        );
        click_copy_chip(app);
        app.update();
        assert_eq!(
            copy_toast_text(app),
            "Copied to clipboard",
            "the copy click shows its toast"
        );
        press(app, KeyCode::Tab);
        wait_for(app, is_drawer_hidden);
        assert_eq!(
            drawer_visibility(app),
            Visibility::Hidden,
            "Tab closes the drawer"
        );
    }

    fn esc_closes_the_leave_prompt_before_menu_and_drawer(app: &mut App) {
        press(app, KeyCode::Escape);
        assert!(is_menu_visible(app), "Esc opens the menu in the room");
        click_leave_in_menu(app);
        wait_for(app, is_leave_prompt_open);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Hidden,
            "the leave press closes the menu"
        );
        press(app, KeyCode::Escape);
        wait_for(app, is_leave_prompt_closed);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Hidden,
            "Esc closes the leave prompt before the menu"
        );
        assert_eq!(
            drawer_visibility(app),
            Visibility::Hidden,
            "Esc closes the leave prompt before the drawer"
        );
        press(app, KeyCode::Escape);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Visible,
            "Esc then opens the menu"
        );
        press(app, KeyCode::Escape);
    }

    fn esc_closes_the_drawer_before_the_menu(app: &mut App) {
        press(app, KeyCode::Tab);
        wait_for(app, is_drawer_fully_open);
        press(app, KeyCode::Escape);
        wait_for(app, is_drawer_hidden);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Hidden,
            "Esc closes the drawer before the menu"
        );
        press(app, KeyCode::Escape);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Visible,
            "Esc then opens the menu"
        );
        press(app, KeyCode::Escape);
    }

    fn the_friends_press_beats_then_opens_the_drawer(app: &mut App) {
        press(app, KeyCode::Escape);
        assert!(is_menu_visible(app), "Esc opens the menu");
        click_friends_in_menu(app);
        app.update();
        assert!(
            friends_button_pressed(app),
            "the Friends button keeps its pressed style during the beat"
        );
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Visible,
            "the beat keeps the menu open"
        );
        assert_eq!(
            drawer_visibility(app),
            Visibility::Hidden,
            "the beat keeps the drawer closed"
        );
        wait_for(app, is_drawer_visible);
        assert_eq!(
            esc_menu_visibility(app),
            Visibility::Hidden,
            "the beat ends by closing the menu"
        );
        assert!(
            !friends_button_pressed(app),
            "the pressed style clears once the beat fires"
        );
        assert!(
            drawer_slide(app) > 0.0,
            "the beat ends by opening the drawer"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn the_drawer_menu_and_copy_flow_run_over_the_fake_network() {
        let orchestrator = testing::FakeNetworkServer::start()
            .await
            .expect("fake network server");
        let network = testing::NetworkArgs {
            network: testing::Network::Fake,
            orchestrator_port: Some(*orchestrator),
        };
        let mut app = default_ui_app(&network);
        app.update();
        let deadline = tokio::time::Instant::now()
            .checked_add(Duration::from_secs(20))
            .unwrap_or_else(tokio::time::Instant::now);
        while is_chip_waiting(&mut app) {
            assert!(
                tokio::time::Instant::now() < deadline,
                "the fake network never filled the address chip"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
            app.update();
        }
        esc_opens_and_closes_the_menu(&mut app);
        the_copy_chip_shows_its_toast(&mut app);
        esc_closes_the_leave_prompt_before_menu_and_drawer(&mut app);
        esc_closes_the_drawer_before_the_menu(&mut app);
        the_friends_press_beats_then_opens_the_drawer(&mut app);
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::Plugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<core_discovery::Command>();
        Plugin::build(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        let mut spinners = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::SpinnerRing>>();
        assert_eq!(spinners.iter(app.world()).count(), 1);
    }
}

#[cfg(test)]
mod finish_plugin_tests {
    use crate::ui;
    use bevy::prelude::Plugin;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        let screen = *app.world().resource::<State<ui::discovery::Screen>>().get();
        assert_eq!(screen, ui::discovery::Screen::Connecting);
    }
}
