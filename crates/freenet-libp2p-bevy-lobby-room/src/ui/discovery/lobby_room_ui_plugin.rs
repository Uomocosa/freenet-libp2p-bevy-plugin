use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy)]
pub struct LobbyRoomUiPlugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for LobbyRoomUiPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
    fn finish(&self, app: &mut App) {
        self.finish_plugin(app);
    }
}

#[atomic_delegates]
impl LobbyRoomUiPlugin {
    pub fn build_plugin(self, app: &mut App) {}
    pub fn finish_plugin(self, app: &mut App) {}
}
#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::freenet;
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

    fn preview_build(app: &mut App) {
        app.add_message::<core_discovery::Command>();
        app.insert_resource(preview_snapshot());
        app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
        app.add_systems(
            Update,
            pin_hover.before(ui::discovery::bevy_systems::animate_hover),
        );
    }

    fn preview_snapshot() -> DiscoveryState {
        let peers: Vec<freenet::FreenetPeer> = (1..=3_u16)
            .map(|port| freenet::FreenetPeer {
                id: freenet::FreenetPeerId::from(format!("peer-{port}").as_str()),
                addr: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            })
            .collect();
        let mut lobby = core_discovery::Lobby::default();
        for (name, room_size) in [("alpha", 8), ("beta", 4)] {
            lobby.insert(
                identity::RoomName::from(name),
                core_discovery::Room {
                    room_size: core_discovery::RoomSize::try_from(room_size)
                        .expect("fixture room size is non-zero"),
                    created_at: time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::new(),
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

    #[test]
    fn default_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("default_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby with two rooms"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("all hovered"),
                    apply: hover_all,
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
                name: identity::RoomName::from("ui-room"),
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
