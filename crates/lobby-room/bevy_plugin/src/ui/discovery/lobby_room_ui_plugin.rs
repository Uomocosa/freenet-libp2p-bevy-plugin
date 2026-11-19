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
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::ui;
    use std::time::Duration;

    use bevy::input::InputPlugin;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::Pressed;
    use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::time;

    #[test]
    fn the_preview_shows_the_friend_names_on_pill_hover() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
        testing::hover_alpha_pill(app.world_mut());
        testing::tick(&mut app, 3);
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            false,
        );
        app.finish();
        testing::tick(&mut app, 5);
        testing::hover_alpha_pill(app.world_mut());
        testing::tick(&mut app, 3);
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomPreviewFriends>>(app.world_mut()),
            0
        );
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Visible)
        );
        let mut images = app
            .world_mut()
            .query_filtered::<Option<&Visibility>, With<ui::discovery::RoomPreviewImage>>();
        let visibility = images.single(app.world()).expect("preview image");
        assert!(!matches!(visibility, Some(Visibility::Hidden)));
    }

    #[test]
    fn test_usage() {
        let mut app = testing::lobby_ui_app(Some(testing::preview_snapshot()), None);
        app.finish();
        testing::tick(&mut app, 3);
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::CreateRoomButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }

    #[test]
    fn the_lobby_spawns_one_preview_and_one_stream_panel() {
        let mut app = testing::lobby_ui_app(
            Some(testing::preview_snapshot()),
            Some(testing::preview_rows()),
        );
        app.add_systems(
            Update,
            testing::bevy_systems::pin_hovered
                .before(ui::discovery::bevy_systems::track_room_preview)
                .before(ui::discovery::bevy_systems::animate_hover),
        );
        app.finish();
        testing::tick(&mut app, 5);
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomPreviewPanel>>(app.world_mut()),
            1
        );
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomStreamPanel>>(app.world_mut()),
            1
        );
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Hidden)
        );
        let row = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
        app.world_mut()
            .entity_mut(row)
            .insert((Hovered(true), testing::PinHovered));
        testing::select_room(&mut app, Some("alpha"));
        testing::tick(&mut app, 3);
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            Some(core_discovery::RoomName::from("alpha")),
            "the pinned hover keeps the selection"
        );
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Visible),
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
        let press = |app: &mut App, room: &str| {
            let row = testing::room_row(app.world_mut(), room).expect("the room row");
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
        testing::tick(&mut app, 4);
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
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
            build: |app: &mut App| {
                testing::lobby_preview_app(
                    app,
                    testing::preview_snapshot(),
                    testing::preview_rows(),
                    true,
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby with three rooms"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("all hovered"),
                    apply: testing::hover_all,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("rooms header hovered"),
                    apply: testing::hover_rooms_header,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room hovered"),
                    apply: |world: &mut World| testing::show_room_preview(world, "alpha"),
                },
                lele_bevy_preview::scene::State {
                    label: String::from("full room selected"),
                    apply: |world: &mut World| testing::show_room_preview(world, "gamma"),
                },
                lele_bevy_preview::scene::State {
                    label: String::from("occupancy pill hovered on alpha"),
                    apply: testing::hover_alpha_pill,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream only"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "alpha");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream and preview of beta"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "alpha");
                        testing::show_room_preview(world, "beta");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream fullscreen"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "alpha");
                        *world.resource_mut::<ui::discovery::StreamDisplay>() =
                            ui::discovery::StreamDisplay::Fullscreen;
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream room full"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "gamma");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream placeholder"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        **world.resource_mut::<ui::discovery::StreamedRoom>() =
                            Some(core_discovery::RoomName::from("alpha"));
                        world.spawn(spectator::Spectated::<testing::FakeReplica<u8, u8>> {
                            slot: spectator::SpectateSlot::Stream,
                            zone: identity::ZoneName::from("alpha"),
                            state: testing::FakeReplica::default(),
                            freshness: spectator::SpectateFreshness::Bootstrapping,
                        });
                    },
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
            build: |app: &mut App| {
                testing::lobby_preview_app(
                    app,
                    testing::preview_snapshot(),
                    testing::preview_rows_long(),
                    true,
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("preview at the top row"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_room_preview(world, "alpha");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("preview at the last row"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_room_preview(world, "room-10");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream with preview at the top row"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "room-09");
                        testing::show_room_preview(world, "alpha");
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("stream with preview at the last row"),
                    apply: |world: &mut World| {
                        testing::clear_room_targets(world);
                        testing::show_stream(world, "alpha");
                        testing::show_room_preview(world, "room-10");
                    },
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
    fn the_yes_answer_leaves_the_room_and_closes_the_prompt() {
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .in_room(testing::joined_room(
                        "alpha",
                        0,
                        core_discovery::JoinPhase::Ready,
                        Vec::new(),
                    ))
                    .build(),
            ),
            None,
        );
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .in_room(testing::joined_room(
                        "alpha",
                        0,
                        core_discovery::JoinPhase::Ready,
                        Vec::new(),
                    ))
                    .build(),
            ),
            None,
        );
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(
            Some(testing::lobby_state("me")),
            Some(core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                2,
                4,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )])),
        );
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(None, None);
        app.add_plugins(InputPlugin);
        app.finish();
        app.update();
        *app.world_mut()
            .resource_mut::<ui::discovery::StreamDisplay>() =
            ui::discovery::StreamDisplay::Fullscreen;
        testing::press_key(&mut app, KeyCode::Escape);
        assert_eq!(
            *app.world().resource::<ui::discovery::StreamDisplay>(),
            ui::discovery::StreamDisplay::Panel
        );
        assert_eq!(
            testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut()),
            Some(Visibility::Hidden)
        );
        testing::press_key(&mut app, KeyCode::Escape);
        assert_eq!(
            testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut()),
            Some(Visibility::Visible)
        );
    }

    #[test]
    fn tab_with_the_menu_open_presses_the_friends_button_and_starts_the_beat() {
        let mut app = testing::lobby_ui_app(None, None);
        app.add_plugins(InputPlugin);
        app.finish();
        app.update();
        testing::press_key(&mut app, KeyCode::Escape);
        assert_eq!(
            testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut()),
            Some(Visibility::Visible)
        );
        testing::press_key(&mut app, KeyCode::Tab);
        assert!(
            testing::find_entity::<ui::discovery::EscMenuFriendsButton>(app.world_mut())
                .is_some_and(|button| app.world().get::<Pressed>(button).is_some())
        );
        assert!(**app.world().resource::<ui::discovery::FriendsPressBeat>() > 0.0);
    }

    #[test]
    fn escape_cancels_a_pending_friends_beat_and_opens_the_menu() {
        let mut app = testing::lobby_ui_app(None, None);
        app.add_plugins(InputPlugin);
        app.finish();
        app.update();
        let button = testing::find_entity::<ui::discovery::EscMenuFriendsButton>(app.world_mut())
            .expect("friends button");
        app.world_mut().entity_mut(button).insert(Pressed);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::FriendsPressBeat>() =
            ui::discovery::MENU_FRIENDS_BEAT_SECS;
        app.update();
        testing::press_key(&mut app, KeyCode::Escape);
        assert!((**app.world().resource::<ui::discovery::FriendsPressBeat>()).abs() < f32::EPSILON);
        assert_eq!(
            testing::marker_visibility::<ui::discovery::EscMenu>(app.world_mut()),
            Some(Visibility::Visible)
        );
    }

    #[test]
    fn a_room_gone_event_records_the_join_error() {
        let mut app = testing::lobby_ui_app(None, None);
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
        let mut app = testing::lobby_ui_app(None, None);
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
        let mut app = testing::lobby_ui_app(None, None);
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
        let mut app = testing::lobby_ui_app(None, None);
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
        let lobby = testing::lobby_with(vec![(
            "alpha",
            testing::room(
                core_discovery::RoomSize::DEFAULT,
                0,
                vec![
                    testing::room_member(
                        "occupant",
                        None,
                        core_discovery::RoomMembership::Occupant,
                        &[],
                    ),
                    testing::room_member(
                        "holder",
                        None,
                        core_discovery::RoomMembership::HistoryHolder,
                        &[],
                    ),
                    testing::room_member(
                        "departed",
                        None,
                        core_discovery::RoomMembership::Departed,
                        &[],
                    ),
                ],
            ),
        )]);
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .with_lobby(lobby)
                    .lobby_fetched()
                    .build(),
            ),
            None,
        );
        app.init_resource::<core_discovery::LobbyRows>();
        app.init_resource::<core_discovery::RoomFriendNames>();
        app.add_systems(
            Update,
            core_discovery::bevy_systems::rebuild_lobby_rows
                .before(ui::discovery::bevy_systems::rebuild_room_list),
        );
        app.finish();
        testing::tick(&mut app, 5);
        let row = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
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
        let mut app = testing::lobby_ui_app(
            Some(testing::lobby_state("me")),
            Some(core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                3,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )])),
        );
        app.finish();
        testing::tick(&mut app, 5);
        let before = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                4,
                8,
                core_discovery::RoomOccupancy::Full,
                0,
            )]);
        app.update();
        let after = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
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
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomButton>>(app.world_mut()),
            0
        );
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomList>>(app.world_mut()),
            1
        );
    }

    #[test]
    fn the_preview_panel_hides_while_the_same_room_streams() {
        let mut app = testing::lobby_ui_app(
            Some(testing::lobby_state("me")),
            Some(core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                1,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )])),
        );
        app.finish();
        testing::tick(&mut app, 5);
        let row = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
        app.world_mut().entity_mut(row).insert(Hovered(true));
        testing::select_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Visible)
        );
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Hidden)
        );
    }

    #[test]
    fn the_stream_panel_follows_the_streamed_room_and_resets_when_it_stops() {
        let mut app = testing::lobby_ui_app(
            Some(testing::lobby_state("me")),
            Some(core_discovery::LobbyRows(vec![
                testing::lobby_row("alpha", 2, 8, core_discovery::RoomOccupancy::HasSpace, 0),
                testing::lobby_row("gamma", 4, 4, core_discovery::RoomOccupancy::Full, 0),
            ])),
        );
        app.finish();
        testing::tick(&mut app, 5);
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
        testing::stream_room(&mut app, Some("alpha"));
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
        testing::stream_room(&mut app, Some("gamma"));
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
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .in_room(testing::joined_room(
                        "alpha",
                        0,
                        core_discovery::JoinPhase::Ready,
                        Vec::new(),
                    ))
                    .build(),
            ),
            None,
        );
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(Some(testing::lobby_state("me")), None);
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(Some(testing::lobby_state("me")), None);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow, CursorIcon::default()))
            .id();
        app.finish();
        testing::tick(&mut app, 5);
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
        let mut app = testing::lobby_ui_app(Some(testing::lobby_state("me")), None);
        app.finish();
        testing::tick(&mut app, 5);
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
                .in_room(testing::joined_room(
                    "alpha",
                    0,
                    core_discovery::JoinPhase::Ready,
                    Vec::new(),
                ))
                .build(),
        );
        testing::tick(&mut app, 5);
        assert_eq!(visibility(&app), Visibility::Inherited);
    }

    #[test]
    fn hovering_a_room_pill_shows_its_friends_tooltip() {
        let mut app = testing::lobby_ui_app(
            Some(testing::lobby_state("me")),
            Some(core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                3,
                4,
                core_discovery::RoomOccupancy::HasSpace,
                1,
            )])),
        );
        app.finish();
        testing::tick(&mut app, 5);
        let row = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
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
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .in_room(testing::joined_room(
                        "alpha",
                        0,
                        core_discovery::JoinPhase::Joining {
                            since: time::UnixTime::EPOCH,
                        },
                        Vec::new(),
                    ))
                    .build(),
            ),
            None,
        );
        app.finish();
        testing::tick(&mut app, 5);
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::JoiningRoomRoot>>(app.world_mut()),
            1
        );
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
        let mut app = testing::lobby_ui_app(
            Some(
                testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                    .in_room(testing::joined_room(
                        "alpha",
                        0,
                        core_discovery::JoinPhase::Ready,
                        Vec::new(),
                    ))
                    .build(),
            ),
            None,
        );
        app.finish();
        let message = String::from("Room alpha no longer exists (all players left)");
        **app.world_mut().resource_mut::<ui::discovery::JoinError>() = Some(message.clone());
        testing::tick(&mut app, 5);
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::JoinErrorRoot>>(app.world_mut()),
            1
        );
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            true,
        );
        app.finish();
        testing::tick(&mut app, 3);
        testing::stream_room(&mut app, Some("alpha"));
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            false,
        );
        app.finish();
        testing::tick(&mut app, 5);
        testing::show_room_preview(app.world_mut(), "alpha");
        testing::tick(&mut app, 3);
        assert_eq!(
            testing::marker_visibility::<ui::discovery::RoomPreviewPanel>(app.world_mut()),
            Some(Visibility::Visible)
        );
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
        let mut app = testing::lobby_ui_app(None, None);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                2,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )]),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
        let row = testing::room_row(app.world_mut(), "alpha").expect("the alpha room row");
        app.world_mut().entity_mut(row).insert(Hovered(true));
        testing::tick(&mut app, 5);
        assert_eq!(
            (**app.world().resource::<ui::discovery::SelectedRoom>()).clone(),
            None,
            "a quarter second of hover is below the preview delay"
        );
        testing::tick(&mut app, 3);
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
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            core_discovery::LobbyRows(vec![
                testing::lobby_row("alpha", 2, 8, core_discovery::RoomOccupancy::HasSpace, 0),
                testing::lobby_row("beta", 1, 8, core_discovery::RoomOccupancy::HasSpace, 0),
            ]),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
        let _ = testing::drained_messages::<core_discovery::Command>(&mut app);
        testing::stream_room(&mut app, Some("alpha"));
        app.update();
        assert_eq!(
            testing::drained_messages::<core_discovery::Command>(&mut app),
            vec![core_discovery::Command::Watch(
                core_discovery::RoomName::from("alpha")
            )],
            "starting a stream watches its room once"
        );
        app.update();
        assert_eq!(
            testing::drained_messages::<core_discovery::Command>(&mut app),
            Vec::new(),
            "the same streamed room is not watched again"
        );
        let row = testing::room_row(app.world_mut(), "beta").expect("the beta room row");
        app.world_mut().entity_mut(row).insert(Hovered(true));
        testing::tick(&mut app, 8);
        assert_eq!(
            testing::drained_messages::<core_discovery::Command>(&mut app),
            vec![core_discovery::Command::Watch(
                core_discovery::RoomName::from("beta")
            )],
            "a hovered room is watched alongside the streamed one"
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
        let mut app = testing::default_ui_app(&network);
        app.update();
        let deadline = tokio::time::Instant::now()
            .checked_add(Duration::from_secs(20))
            .unwrap_or_else(tokio::time::Instant::now);
        while testing::marker_text::<ui::direct_dial::OwnAddressText>(app.world_mut())
            .is_none_or(|label| label.contains("waiting"))
        {
            assert!(
                tokio::time::Instant::now() < deadline,
                "the fake network never filled the address chip"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
            app.update();
        }
        testing::app::drive_esc_menu_open_and_close::drive_esc_menu_open_and_close(&mut app)
            .expect("the esc menu opens and closes");
        testing::app::drive_copy_chip_toast::drive_copy_chip_toast(
            &mut app,
            ui::constants::DRAWER_SLIDE_SECS,
        )
        .expect("the copy chip shows its toast");
        testing::app::drive_esc_closes_leave_prompt::drive_esc_closes_leave_prompt(&mut app)
            .expect("Esc closes the leave prompt first");
        testing::app::drive_esc_closes_drawer_before_menu::drive_esc_closes_drawer_before_menu(
            &mut app,
            ui::constants::DRAWER_SLIDE_SECS,
        )
        .expect("Esc closes the drawer before the menu");
        testing::app::drive_friends_press_beat::drive_friends_press_beat(&mut app)
            .expect("the friends press beats then opens the drawer");
    }

    #[test]
    fn the_friend_badge_appears_after_a_roll_and_hides_when_the_last_friend_leaves() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )));
        testing::lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                2,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )]),
            true,
        );
        app.finish();
        testing::tick(&mut app, 5);
        assert_eq!(
            testing::count_matching::<(), With<ui::discovery::RoomFriendSegment>>(app.world_mut()),
            0
        );
        *app.world_mut().resource_mut::<core_discovery::LobbyRows>() =
            core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                2,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                1,
            )]);
        let mut appeared = false;
        for _ in 0..40 {
            app.update();
            if testing::count_matching::<(), With<ui::discovery::RoomFriendSegment>>(
                app.world_mut(),
            ) == 1
            {
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
            core_discovery::LobbyRows(vec![testing::lobby_row(
                "alpha",
                2,
                8,
                core_discovery::RoomOccupancy::HasSpace,
                0,
            )]);
        let mut vanished = false;
        for _ in 0..40 {
            app.update();
            if testing::count_matching::<(), With<ui::discovery::RoomFriendSegment>>(
                app.world_mut(),
            ) == 0
            {
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
