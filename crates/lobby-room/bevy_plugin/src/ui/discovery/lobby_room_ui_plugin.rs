use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::App;
use bevy::prelude::*;
use freenet_libp2p_bevy_core::replica_sync;
use freenet_libp2p_bevy_core::ui as core_ui;

#[derive(Debug, Default, Clone, Copy)]
pub struct LobbyRoomUiPlugin;

impl bevy::prelude::Plugin for LobbyRoomUiPlugin {
    fn build(&self, app: &mut App) {
        use ui::discovery::bevy_systems as systems;
        use ui::discovery::{EscMenuState, JoinError, LeaveDialog, Screen};

        app.add_plugins(ui::roster::UiPlugin);
        app.add_plugins(ui::roll::UiPlugin);
        app.add_plugins(core_ui::floating::UiPlugin);
        app.init_resource::<JoinError>();
        app.init_resource::<EscMenuState>();
        app.init_resource::<ui::discovery::FriendsPressBeat>();
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.init_resource::<ui::discovery::PeekedRoomFriends>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.init_resource::<ui::discovery::StreamedRoom>();
        app.init_resource::<ui::discovery::StreamDisplay>();
        app.add_message::<ui::discovery::CloseStream>();
        app.init_resource::<ButtonInput<KeyCode>>();
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
                systems::show_room_bubble_tooltip
                    .after(systems::rebuild_room_list)
                    .run_if(in_state(Screen::Lobby)),
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
                        systems::sync_peeked_room_friends.after(systems::show_room_bubble_tooltip),
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
        app.add_message::<core_discovery::Event>();
        app.add_message::<replica_sync::JoinFailure>();
        app.add_systems(
            OnExit(ui::discovery::Screen::Lobby),
            ui::discovery::bevy_systems::reset_peeked_room_friends,
        );
        app.configure_sets(
            Update,
            ui::roll::RollSet::Drive.after(ui::discovery::bevy_systems::rebuild_room_list),
        );
        app.add_systems(
            Update,
            ui::discovery::bevy_systems::sync_room_friend_segment
                .after(ui::roll::RollSet::Drive)
                .run_if(in_state(ui::discovery::Screen::Lobby)),
        );
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
    use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};
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
        preview_build_ui_with(app, rows);
        app.add_plugins(ui::discovery::PreviewFriendsCardPlugin);
    }

    fn preview_build_ui_with(app: &mut App, rows: core_discovery::LobbyRows) {
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
            pin_pill_hover.before(ui::discovery::bevy_systems::show_room_bubble_tooltip),
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
                friends: 2,
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

    #[derive(Component)]
    struct PinPillHover;

    fn pin_pill_hover(mut commands: Commands, pinned: Query<Entity, With<PinPillHover>>) {
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

    fn pin_alpha_pill(world: &mut World) {
        let mut rows = world.query::<(Entity, &ui::discovery::RoomButton)>();
        let Some(row) = rows
            .iter(world)
            .find(|(_entity, button)| button.as_str() == "alpha")
            .map(|(entity, _button)| entity)
        else {
            return;
        };
        let mut pills = world.query::<(Entity, &ChildOf, &ui::discovery::RoomBubble)>();
        let alpha_pills: Vec<Entity> = pills
            .iter(world)
            .filter(|(_entity, link, _bubble)| link.parent() == row)
            .map(|(entity, _link, _bubble)| entity)
            .collect();
        for pill in alpha_pills {
            world.entity_mut(pill).insert((Hovered(true), PinPillHover));
        }
    }

    fn hover_alpha_pill(world: &mut World) {
        select_room_value(world, "alpha");
        pin_alpha_pill(world);
        **world.resource_mut::<ui::discovery::HoveredOccupancyPill>() =
            Some(core_discovery::RoomName::from("alpha"));
        world.insert_resource(core_discovery::RoomFriendNames(
            std::collections::BTreeMap::from([(
                core_discovery::RoomName::from("alpha"),
                vec![
                    identity::Username::from("alice"),
                    identity::Username::from("bob"),
                ],
            )]),
        ));
    }

    #[test]
    fn the_preview_shows_the_friend_names_on_pill_hover() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        preview_build(&mut app);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        hover_alpha_pill(app.world_mut());
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            (**app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>())
            .clone(),
            Some(core_discovery::RoomName::from("alpha"))
        );
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            Some(core_discovery::RoomName::from("alpha"))
        );
        let mut overlays = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomPreviewFriends>>();
        let overlay = overlays.single(app.world()).expect("friends overlay");
        assert_eq!(
            *app.world()
                .get::<Visibility>(overlay)
                .expect("overlay visibility"),
            Visibility::Inherited
        );
        let mut texts = app.world_mut().query_filtered::<&Text, ()>();
        let labels: Vec<String> = texts
            .iter(app.world())
            .filter(|text| ["friends", "alice", "bob"].contains(&text.as_str()))
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(labels, vec!["friends", "alice", "bob"]);
    }

    #[test]
    fn without_the_friends_card_plugin_the_preview_image_stays_visible() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        preview_build_ui_with(&mut app, preview_rows());
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        hover_alpha_pill(app.world_mut());
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(count::<ui::discovery::RoomPreviewFriends>(&mut app), 0);
        assert_eq!(preview_visibility(&mut app), Visibility::Visible);
        let mut images = app
            .world_mut()
            .query_filtered::<Option<&Visibility>, With<ui::discovery::RoomPreviewImage>>();
        let visibility = images.single(app.world()).expect("preview image");
        assert!(!matches!(visibility, Some(Visibility::Hidden)));
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
    fn double_clicking_a_room_row_joins_it_once() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
        app.add_message::<core_discovery::Command>();
        preview_build(&mut app);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let press = |app: &mut App, room: &str| {
            let row = {
                let mut rows = app
                    .world_mut()
                    .query::<(Entity, &ui::discovery::RoomButton)>();
                rows.iter(app.world())
                    .find(|(_entity, button)| button.as_str() == room)
                    .map(|(entity, _button)| entity)
                    .expect("the room row")
            };
            app.world_mut().entity_mut(row).remove::<Pressed>();
            app.world_mut().entity_mut(row).insert(Pressed);
            app.update();
        };
        let mut cursor = app
            .world()
            .resource::<Messages<core_discovery::Command>>()
            .get_cursor();
        let mut joins = |app: &mut App| -> Vec<core_discovery::Command> {
            let messages = app.world().resource::<Messages<core_discovery::Command>>();
            cursor
                .read(messages)
                .filter(|command| matches!(command, core_discovery::Command::Join(_)))
                .cloned()
                .collect()
        };
        press(&mut app, "gamma");
        press(&mut app, "gamma");
        assert!(
            joins(&mut app).is_empty(),
            "a full room never joins on a double click"
        );
        press(&mut app, "alpha");
        press(&mut app, "beta");
        assert!(
            joins(&mut app).is_empty(),
            "two presses on different rows do not join"
        );
        press(&mut app, "alpha");
        for _ in 0..4 {
            app.update();
        }
        press(&mut app, "alpha");
        assert!(
            joins(&mut app).is_empty(),
            "two presses outside the double-click window do not join"
        );
        press(&mut app, "alpha");
        assert_eq!(
            joins(&mut app),
            vec![core_discovery::Command::Join(
                core_discovery::RoomName::from("alpha")
            )],
            "the double click joins the room once"
        );
        press(&mut app, "alpha");
        assert_eq!(
            joins(&mut app),
            Vec::new(),
            "the press after a join starts a new double click"
        );
    }

    #[test]
    fn the_stream_image_letterboxes_into_its_viewport() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        preview_build(&mut app);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let viewport = {
            let mut viewports = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomStreamViewport>>();
            viewports.single(app.world()).expect("stream viewport")
        };
        let fitted = |app: &mut App| -> (f32, f32) {
            let mut images = app
                .world_mut()
                .query_filtered::<&Node, With<ui::discovery::RoomStreamImage>>();
            let node = images.single(app.world()).expect("stream image node");
            let (Val::Px(width), Val::Px(height)) = (node.width, node.height) else {
                panic!("the fitted image uses pixel sizes");
            };
            (width, height)
        };
        assert_eq!(
            fitted(&mut app),
            (0.0, 0.0),
            "an unmeasured panel keeps the spawn size"
        );
        app.world_mut()
            .entity_mut(viewport)
            .insert(bevy::ui::ComputedNode {
                size: Vec2::new(640.0, 300.0),
                ..bevy::ui::ComputedNode::default()
            });
        app.update();
        assert_eq!(
            fitted(&mut app),
            (480.0, 300.0),
            "a tall panel letterboxes the sides"
        );
        app.world_mut()
            .entity_mut(viewport)
            .insert(bevy::ui::ComputedNode {
                size: Vec2::new(640.0, 600.0),
                ..bevy::ui::ComputedNode::default()
            });
        app.update();
        assert_eq!(
            fitted(&mut app),
            (640.0, 400.0),
            "a short panel letterboxes the top and bottom"
        );
        app.world_mut()
            .entity_mut(viewport)
            .insert(bevy::ui::ComputedNode {
                size: Vec2::new(640.0, 400.0),
                padding: bevy::sprite::BorderRect {
                    min_inset: Vec2::new(12.0, 64.0),
                    max_inset: Vec2::new(12.0, 12.0),
                },
                ..bevy::ui::ComputedNode::default()
            });
        app.update();
        let (width, height) = fitted(&mut app);
        assert!(
            (width - 518.4).abs() < 0.01,
            "the padding is excluded from the fitted area"
        );
        assert!((height - 324.0).abs() < f32::EPSILON);
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
                    label: String::from("occupancy pill hovered on alpha"),
                    apply: hover_alpha_pill,
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

    #[test]
    fn pick_screen_routes_each_discovery_state_to_its_screen() {
        let screen_after = |state: Option<DiscoveryState>, join_error: Option<&str>| {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, StatesPlugin));
            app.add_message::<core_discovery::Command>();
            if let Some(state) = state {
                app.insert_resource(state);
            }
            app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
            app.finish();
            if let Some(message) = join_error {
                **app.world_mut().resource_mut::<ui::discovery::JoinError>() =
                    Some(String::from(message));
            }
            for _ in 0..5 {
                app.update();
            }
            *app.world().resource::<State<ui::discovery::Screen>>().get()
        };
        let in_room = |phase| {
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build()
        };
        assert_eq!(screen_after(None, None), ui::discovery::Screen::Connecting);
        assert_eq!(
            screen_after(
                Some(
                    testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                        .lobby_fetched()
                        .build()
                ),
                None
            ),
            ui::discovery::Screen::Lobby
        );
        assert_eq!(
            screen_after(
                Some(in_room(core_discovery::JoinPhase::Joining {
                    since: time::UnixTime::EPOCH,
                })),
                None
            ),
            ui::discovery::Screen::JoiningRoom
        );
        assert_eq!(
            screen_after(Some(in_room(core_discovery::JoinPhase::Ready)), None),
            ui::discovery::Screen::InRoom
        );
        assert_eq!(
            screen_after(
                Some(in_room(core_discovery::JoinPhase::Ready)),
                Some("gone")
            ),
            ui::discovery::Screen::JoinFailed
        );
    }

    #[test]
    fn the_yes_answer_leaves_the_room_and_closes_the_prompt() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        app.world_mut()
            .resource_mut::<NextState<ui::discovery::LeaveDialog>>()
            .set(ui::discovery::LeaveDialog::Open);
        app.update();
        app.world_mut()
            .spawn((ui::discovery::LeaveYesButton, Pressed));
        app.update();
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(
            cursor.read(messages).cloned().collect::<Vec<_>>(),
            vec![core_discovery::Command::Leave]
        );
        assert!(matches!(
            *app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Closed)
        ));
    }

    #[test]
    fn the_no_answer_closes_the_prompt_without_leaving() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        app.world_mut()
            .resource_mut::<NextState<ui::discovery::LeaveDialog>>()
            .set(ui::discovery::LeaveDialog::Open);
        app.update();
        app.world_mut()
            .spawn((ui::discovery::LeaveNoButton, Pressed));
        app.update();
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(
            cursor
                .read(messages)
                .filter(|command| matches!(command, core_discovery::Command::Leave))
                .count(),
            0
        );
        assert!(matches!(
            *app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Closed)
        ));
    }

    #[test]
    fn a_room_without_friends_has_no_friend_suffix_in_its_tooltip() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let mut texts = app.world_mut().query::<&Text>();
        let labels: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert!(
            labels.iter().any(|label| label == "2 of 4 players"),
            "a room with no friends shows only its occupancy"
        );
    }

    #[test]
    fn escape_leaves_fullscreen_before_opening_the_menu() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_plugins(InputPlugin);
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.update();
        *app.world_mut()
            .resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
        press(&mut app, KeyCode::Escape);
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Panel
        );
        assert_eq!(esc_menu_visibility(&mut app), Visibility::Hidden);
        press(&mut app, KeyCode::Escape);
        assert_eq!(esc_menu_visibility(&mut app), Visibility::Visible);
    }

    #[test]
    fn tab_with_the_menu_open_presses_the_friends_button_and_starts_the_beat() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_plugins(InputPlugin);
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.update();
        press(&mut app, KeyCode::Escape);
        assert_eq!(esc_menu_visibility(&mut app), Visibility::Visible);
        press(&mut app, KeyCode::Tab);
        assert!(friends_button_pressed(&mut app));
        assert!(**app.world().resource::<ui::discovery::FriendsPressBeat>() > 0.0);
    }

    #[test]
    fn escape_cancels_a_pending_friends_beat_and_opens_the_menu() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_plugins(InputPlugin);
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.update();
        let button = {
            let mut buttons = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::EscMenuFriendsButton>>();
            buttons.single(app.world()).expect("friends button")
        };
        app.world_mut().entity_mut(button).insert(Pressed);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::FriendsPressBeat>() =
            ui::discovery::MENU_FRIENDS_BEAT_SECS;
        app.update();
        press(&mut app, KeyCode::Escape);
        assert!((**app.world().resource::<ui::discovery::FriendsPressBeat>()).abs() < f32::EPSILON);
        assert_eq!(esc_menu_visibility(&mut app), Visibility::Visible);
    }

    #[test]
    fn a_room_gone_event_records_the_join_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.world_mut()
            .resource_mut::<Messages<core_discovery::Event>>()
            .write(core_discovery::Event::RoomGone(
                core_discovery::RoomName::from("alpha"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }

    #[test]
    fn a_history_unavailable_failure_records_the_history_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.world_mut()
            .resource_mut::<Messages<replica_sync::JoinFailure>>()
            .write(replica_sync::JoinFailure::HistoryUnavailable(
                identity::ZoneName::from("beta"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from("Couldn't get room history"))
        );
    }

    #[test]
    fn a_history_unavailable_event_records_the_history_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.world_mut()
            .resource_mut::<Messages<core_discovery::Event>>()
            .write(core_discovery::Event::HistoryUnavailable(
                core_discovery::RoomName::from("beta"),
            ));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from("Couldn't get room history"))
        );
    }

    #[test]
    fn a_joined_event_clears_the_join_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        **app.world_mut().resource_mut::<ui::discovery::JoinError>() =
            Some(String::from("an earlier join error"));
        app.world_mut()
            .resource_mut::<Messages<core_discovery::Event>>()
            .write(core_discovery::Event::Joined(
                core_discovery::RoomName::from("alpha"),
            ));
        app.update();
        assert_eq!(**app.world().resource::<ui::discovery::JoinError>(), None);
    }

    #[test]
    fn the_row_count_covers_occupant_members_only() {
        let member = |peer: &str, membership| {
            (
                identity::PeerId::from(peer),
                core_discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(peer),
                        addrs: Vec::new(),
                    },
                    display_name: None,
                    updated_at: time::UnixTime::EPOCH,
                    membership,
                },
            )
        };
        let mut lobby = core_discovery::Lobby::default();
        lobby.insert(
            core_discovery::RoomName::from("alpha"),
            core_discovery::Room {
                room_size: core_discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::from([
                    member("occupant", core_discovery::RoomMembership::Occupant),
                    member("holder", core_discovery::RoomMembership::HistoryHolder),
                    member("departed", core_discovery::RoomMembership::Departed),
                ]),
            },
        );
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .lobby_fetched()
                .build(),
        );
        app.init_resource::<core_discovery::LobbyRows>();
        app.init_resource::<core_discovery::RoomFriendNames>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.add_systems(
            Update,
            core_discovery::bevy_systems::rebuild_lobby_rows
                .before(ui::discovery::bevy_systems::rebuild_room_list),
        );
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let row = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        let mut found: Vec<(usize, u32)> = numbers
            .iter(app.world())
            .filter(|number| number.pill == row)
            .map(|number| (number.slot, number.value))
            .collect();
        found.sort_unstable();
        assert_eq!(found, vec![(0, 1), (1, 8)]);
    }

    #[test]
    fn a_value_change_reuses_the_row_then_clearing_the_lobby_despawns_it() {
        let row = |occupants, occupancy| core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy,
            friends: 0,
        };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![row(
            3,
            core_discovery::RoomOccupancy::HasSpace,
        )]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let before = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(vec![row(4, core_discovery::RoomOccupancy::Full)]);
        app.update();
        let after = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        assert_eq!(after, before, "the row entity survives a value change");
        assert_eq!(
            app.world()
                .get::<ui::roll::RollMachine>(before)
                .expect("roll machine")
                .target,
            vec![4, 8, 0]
        );
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(Vec::new());
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 0);
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
    }

    #[test]
    fn the_preview_panel_hides_while_the_same_room_streams() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 1,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let row = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        app.world_mut().entity_mut(row).insert(Hovered(true));
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert_eq!(preview_visibility(&mut app), Visibility::Visible);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert_eq!(preview_visibility(&mut app), Visibility::Hidden);
    }

    #[test]
    fn the_stream_panel_follows_the_streamed_room_and_resets_when_it_stops() {
        let row =
            |name: &str, occupants: usize, room_size: u16, occupancy| core_discovery::LobbyRow {
                name: core_discovery::RoomName::from(name),
                occupants,
                room_size: core_discovery::RoomSize::try_from(room_size).expect("room size"),
                occupancy,
                friends: 0,
            };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![
            row("alpha", 2, 8, core_discovery::RoomOccupancy::HasSpace),
            row("gamma", 4, 4, core_discovery::RoomOccupancy::Full),
        ]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        *app.world_mut()
            .resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
        app.update();
        let mut panels = app
            .world_mut()
            .query_filtered::<&Visibility, With<ui::discovery::RoomStreamPanel>>();
        assert_eq!(
            *panels.single(app.world()).expect("stream panel"),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Panel
        );
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert_eq!(
            *panels.single(app.world()).expect("stream panel"),
            Visibility::Visible
        );
        let mut names = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamName>>();
        assert_eq!(names.single(app.world()).expect("name").as_str(), "alpha");
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamOccupancyText>>();
        assert_eq!(
            texts.single(app.world()).expect("occupancy").as_str(),
            "2/8"
        );
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("gamma"));
        app.update();
        let mut names = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamName>>();
        assert_eq!(names.single(app.world()).expect("name").as_str(), "gamma");
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, With<ui::discovery::RoomStreamOccupancyText>>();
        assert_eq!(
            texts.single(app.world()).expect("occupancy").as_str(),
            "4/4"
        );
    }

    #[test]
    fn pressing_the_leave_door_opens_the_leave_prompt_once() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let door = {
            let mut doors = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::LeaveDoor>>();
            doors.single(app.world()).expect("leave door")
        };
        app.world_mut().entity_mut(door).insert(Pressed);
        app.update();
        assert!(matches!(
            *app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Open)
        ));
        app.update();
        app.world_mut().entity_mut(door).remove::<Pressed>();
        app.world_mut().entity_mut(door).insert(Pressed);
        app.update();
        assert!(!matches!(
            *app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Open)
        ));
    }

    #[test]
    fn hovering_the_rooms_header_shows_the_search_tooltip() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let indicator = {
            let mut indicators = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
            indicators.single(app.world()).expect("search indicator")
        };
        let tooltip = {
            let mut tooltips = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomSearchTooltip>>();
            tooltips.single(app.world()).expect("search tooltip")
        };
        let visibility = |app: &App| {
            *app.world()
                .get::<Visibility>(tooltip)
                .expect("search tooltip")
        };
        assert_eq!(visibility(&app), Visibility::Hidden);
        app.world_mut().entity_mut(indicator).insert(Hovered(true));
        app.update();
        assert_eq!(visibility(&app), Visibility::Visible);
        app.world_mut().entity_mut(indicator).insert(Hovered(false));
        app.update();
        assert_eq!(visibility(&app), Visibility::Hidden);
    }

    #[test]
    fn the_room_name_input_shows_the_text_cursor_on_hover() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow, CursorIcon::default()))
            .id();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let input = {
            let mut inputs = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomNameInput>>();
            inputs.single(app.world()).expect("room name input")
        };
        app.world_mut().entity_mut(input).insert(Hovered(true));
        app.update();
        assert_eq!(
            *app.world().get::<CursorIcon>(window).expect("cursor"),
            CursorIcon::System(SystemCursorIcon::Text)
        );
        app.world_mut().entity_mut(input).insert(Hovered(false));
        app.update();
        assert_eq!(
            *app.world().get::<CursorIcon>(window).expect("cursor"),
            CursorIcon::default()
        );
    }

    #[test]
    fn the_menu_leave_button_shows_only_in_a_room() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let button = {
            let mut buttons = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::EscMenuLeaveButton>>();
            buttons.single(app.world()).expect("leave button")
        };
        let visibility = |app: &App| *app.world().get::<Visibility>(button).expect("leave button");
        assert_eq!(visibility(&app), Visibility::Hidden);
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(visibility(&app), Visibility::Inherited);
    }

    #[test]
    fn hovering_a_room_pill_shows_its_friends_tooltip() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 3,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 1,
        }]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let row = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        let pill = {
            let mut pills = app
                .world_mut()
                .query::<(Entity, &ChildOf, &ui::discovery::RoomBubble)>();
            pills
                .iter(app.world())
                .find(|(_entity, link, _bubble)| link.parent() == row)
                .map(|(entity, _link, _bubble)| entity)
                .expect("the alpha room pill")
        };
        let tooltip = {
            let mut tooltips = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomBubbleTooltip>>();
            tooltips.single(app.world()).expect("bubble tooltip")
        };
        let visibility = |app: &App| {
            *app.world()
                .get::<Visibility>(tooltip)
                .expect("bubble tooltip")
        };
        assert_eq!(visibility(&app), Visibility::Hidden);
        app.world_mut().entity_mut(pill).insert(Hovered(true));
        app.update();
        assert_eq!(visibility(&app), Visibility::Visible);
        assert_eq!(
            app.world()
                .get::<Text>(tooltip)
                .expect("tooltip text")
                .as_str(),
            "3 of 4 players | 1 friend"
        );
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            Some(core_discovery::RoomName::from("alpha"))
        );
        app.world_mut().entity_mut(pill).insert(Hovered(false));
        app.update();
        assert_eq!(visibility(&app), Visibility::Hidden);
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            None
        );
    }

    #[test]
    fn the_joining_screen_names_the_room_from_the_state() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Joining {
                        since: time::UnixTime::EPOCH,
                    },
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(count::<ui::discovery::JoiningRoomRoot>(&mut app), 1);
        let mut texts = app.world_mut().query::<&Text>();
        let labels: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert!(
            labels.iter().any(|label| label == "Joining room alpha..."),
            "the joining screen names the room: {labels:?}"
        );
    }

    #[test]
    fn the_join_failed_screen_shows_the_recorded_error() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        let message = String::from("Room alpha no longer exists (all players left)");
        **app.world_mut().resource_mut::<ui::discovery::JoinError>() = Some(message.clone());
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(count::<ui::discovery::JoinErrorRoot>(&mut app), 1);
        let mut texts = app.world_mut().query::<&Text>();
        let labels: Vec<String> = texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert!(
            labels.contains(&message),
            "the join failed screen shows the recorded error: {labels:?}"
        );
    }

    #[test]
    fn the_stream_close_button_clears_the_streamed_room() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.insert_resource(preview_rows());
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..3 {
            app.update();
        }
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert!(
            (**app.world().resource::<ui::discovery::StreamedRoom>()).is_some(),
            "the stream is running before the press"
        );
        let button = {
            let mut buttons = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomStreamCloseButton>>();
            buttons.single(app.world()).expect("stream close button")
        };
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        assert!(
            (**app.world().resource::<ui::discovery::StreamedRoom>()).is_none(),
            "the close press clears the streamed room"
        );
    }

    #[test]
    fn the_preview_never_prints_a_member_name() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        preview_build_ui_with(&mut app, preview_rows());
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        select_room(app.world_mut());
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(preview_visibility(&mut app), Visibility::Visible);
        let panel = {
            let mut panels = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::RoomPreviewPanel>>();
            panels.single(app.world()).expect("preview panel")
        };
        let mut stack = vec![panel];
        let mut texts = Vec::new();
        while let Some(entity) = stack.pop() {
            if let Some(text) = app.world().get::<Text>(entity) {
                texts.push(text.as_str().to_string());
            }
            if let Some(children) = app.world().get::<Children>(entity) {
                stack.extend(children.iter());
            }
        }
        assert!(
            texts.is_empty(),
            "the preview is image-only and never prints a member name: {texts:?}"
        );
    }

    #[test]
    fn the_connecting_spinner_rotates() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        app.update();
        app.update();
        let mut rings = app
            .world_mut()
            .query_filtered::<&UiTransform, With<ui::discovery::SpinnerRing>>();
        let rotation = rings.single(app.world()).expect("spinner ring").rotation;
        assert_ne!(
            rotation,
            Rot2::IDENTITY,
            "the connecting spinner has turned after two frames"
        );
    }

    #[test]
    fn hovering_a_room_row_for_the_delay_opens_its_preview_and_leaving_closes_it() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 2,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 0,
        }]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let row = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "alpha")
                .map(|(entity, _button)| entity)
                .expect("the alpha room row")
        };
        app.world_mut().entity_mut(row).insert(Hovered(true));
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            None,
            "a quarter second of hover is below the preview delay"
        );
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            Some(core_discovery::RoomName::from("alpha")),
            "the hover delay opens the preview"
        );
        app.world_mut().entity_mut(row).remove::<Hovered>();
        app.update();
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            None,
            "leaving the row closes the preview"
        );
    }

    #[test]
    fn watching_a_streamed_room_emits_one_watch_command_and_a_new_hover_watches_both() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![
            core_discovery::LobbyRow {
                name: core_discovery::RoomName::from("alpha"),
                occupants: 2,
                room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 0,
            },
            core_discovery::LobbyRow {
                name: core_discovery::RoomName::from("beta"),
                occupants: 1,
                room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                occupancy: core_discovery::RoomOccupancy::HasSpace,
                friends: 0,
            },
        ]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let _ = app
            .world_mut()
            .resource_mut::<Messages<core_discovery::Command>>()
            .drain()
            .count();
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            Some(core_discovery::RoomName::from("alpha"));
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<core_discovery::Command>>()
                .drain()
                .collect::<Vec<_>>(),
            vec![core_discovery::Command::Watch(
                core_discovery::RoomName::from("alpha")
            )],
            "starting a stream watches its room once"
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<core_discovery::Command>>()
                .drain()
                .count(),
            0,
            "the same streamed room is not watched again"
        );
        let row = {
            let mut rows = app
                .world_mut()
                .query::<(Entity, &ui::discovery::RoomButton)>();
            rows.iter(app.world())
                .find(|(_entity, button)| button.as_str() == "beta")
                .map(|(entity, _button)| entity)
                .expect("the beta room row")
        };
        app.world_mut().entity_mut(row).insert(Hovered(true));
        for _ in 0..8 {
            app.update();
        }
        assert_eq!(
            app.world_mut()
                .resource_mut::<Messages<core_discovery::Command>>()
                .drain()
                .collect::<Vec<_>>(),
            vec![core_discovery::Command::Watch(
                core_discovery::RoomName::from("beta")
            )],
            "a hovered room is watched alongside the streamed one"
        );
    }

    #[test]
    fn the_friend_badge_appears_after_a_roll_and_hides_when_the_last_friend_leaves() {
        let row = |occupants: usize, friends: usize| core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        };
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        app.add_message::<core_discovery::Command>();
        app.insert_resource(core_discovery::LobbyRows(vec![row(2, 0)]));
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .lobby_fetched()
                .build(),
        );
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.finish();
        for _ in 0..5 {
            app.update();
        }
        let mut segments = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomFriendSegment>>();
        assert_eq!(segments.iter(app.world()).count(), 0);
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(vec![row(2, 1)]);
        let mut appeared = false;
        for _ in 0..40 {
            app.update();
            let mut segments = app
                .world_mut()
                .query_filtered::<(), With<ui::discovery::RoomFriendSegment>>();
            if segments.iter(app.world()).count() == 1 {
                appeared = true;
                break;
            }
        }
        assert!(
            appeared,
            "the friend badge appears once its roll reveals it"
        );
        let bubble = {
            let mut bubbles = app
                .world_mut()
                .query_filtered::<(Entity, &Children), With<ui::discovery::RoomBubble>>();
            bubbles
                .single(app.world())
                .map(|(_entity, children)| *children.first().expect("a first child"))
                .expect("one room bubble")
        };
        assert!(
            app.world()
                .get::<ui::discovery::RoomFriendSegment>(bubble)
                .is_some(),
            "the friend badge sits before the occupancy numbers"
        );
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(vec![row(2, 0)]);
        let mut vanished = false;
        for _ in 0..40 {
            app.update();
            let mut segments = app
                .world_mut()
                .query_filtered::<(), With<ui::discovery::RoomFriendSegment>>();
            if segments.iter(app.world()).count() == 0 {
                vanished = true;
                break;
            }
        }
        assert!(
            vanished,
            "the friend badge hides once the last friend has rolled away"
        );
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use crate::discovery as core_discovery;
    use crate::testing;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::Plugin;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::time;
    use std::time::Duration;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.add_message::<core_discovery::Command>();
        Plugin::build(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        let mut spinners = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::SpinnerRing>>();
        assert_eq!(spinners.iter(app.world()).count(), 1);
    }

    #[test]
    fn hovering_the_leave_door_opens_it_fully() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
        app.add_message::<core_discovery::Command>();
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .in_room(core_discovery::JoinedRoom {
                    name: core_discovery::RoomName::from("alpha"),
                    created_at: time::UnixTime::EPOCH,
                    phase: core_discovery::JoinPhase::Ready,
                    members: core_discovery::Members::new(),
                    how_entered: core_discovery::HowRoomWasEntered::Created,
                })
                .build(),
        );
        for _ in 0..5 {
            app.update();
        }
        let door = {
            let mut doors = app
                .world_mut()
                .query_filtered::<Entity, With<ui::discovery::LeaveDoor>>();
            doors.single(app.world()).expect("leave door")
        };
        app.world_mut().entity_mut(door).insert(Hovered(true));
        for _ in 0..10 {
            app.update();
        }
        let mut openings = app.world_mut().query::<&ui::discovery::DoorOpening>();
        let opening = openings.single(app.world()).map(|opening| **opening).ok();
        assert_eq!(opening, Some(1.0));
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
