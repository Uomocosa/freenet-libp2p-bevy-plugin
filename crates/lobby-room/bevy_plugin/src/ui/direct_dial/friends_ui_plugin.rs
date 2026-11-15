use crate::ui;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::App;
use bevy::prelude::*;
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

#[derive(Debug, Default, Clone, Copy)]
pub struct FriendsUiPlugin;

impl bevy::prelude::Plugin for FriendsUiPlugin {
    fn build(&self, app: &mut App) {
        use ui::direct_dial::bevy_systems as friends;
        app.init_resource::<ui::direct_dial::FriendsDrawerState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<bevy::clipboard::Clipboard>();
        app.add_message::<core_direct_dial::FriendAddressSubmitted>();
        app.add_systems(
            Startup,
            (
                friends::spawn_friends_drawer,
                friends::spawn_drawer_sections,
            )
                .chain(),
        );
        app.add_systems(Update, friends::close_drawer_on_press);
        app.add_systems(Update, friends::rebuild_friend_list);
        app.add_systems(Update, friends::refresh_friend_counts);
        app.add_systems(Update, friends::gate_add_friend);
        app.add_systems(
            Update,
            (
                friends::handle_add_friend,
                friends::show_friend_address_error,
            )
                .chain(),
        );
        app.add_systems(Update, friends::handle_remove_friend);
        app.add_systems(Update, friends::refresh_own_address);
        app.add_systems(Update, friends::handle_copy_own_address);
        app.add_systems(Update, friends::animate_friends_drawer);
        app.add_systems(Update, friends::animate_copy_toast);
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    use super::FriendsUiPlugin;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::time;

    fn app_with_state() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.add_message::<core_direct_dial::FriendAddressSubmitted>();
        app.insert_resource(core_direct_dial::state_machine::DialState::from_peer_book(
            &core_direct_dial::PeerBook::default(),
            time::UnixTime::from_secs(10),
        ));
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        app.init_resource::<core_direct_dial::FriendsView>();
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_state();
        app.add_plugins(FriendsUiPlugin);
        app.update();
        let mut drawers = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendsDrawer>>();
        assert_eq!(drawers.iter(app.world()).count(), 1);
        let mut states = app
            .world_mut()
            .query_filtered::<&ui::direct_dial::FriendsDrawerState, ()>();
        assert_eq!(states.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        use freenet_libp2p_bevy_core::identity;
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.insert_resource(core_direct_dial::OwnShareAddress(Some(
            core_direct_dial::OwnAddress {
                id: identity::PeerId::from(
                    "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
                ),
                short: String::from("12D3Koo...99RC"),
                full: Some(String::from(
                    "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
                )),
            },
        )));
        app.init_resource::<core_direct_dial::FriendsView>();
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        app.add_systems(Startup, spawn_mock_screen);
        app.add_plugins(FriendsUiPlugin);
    }

    fn spawn_mock_screen(mut commands: Commands) {
        commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ui::constants::ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(ui::discovery::PANEL_COLOR),
                children![Text::new("the screen behind the drawer")],
            )],
        ));
    }
    fn preview_view() -> core_direct_dial::FriendsView {
        core_direct_dial::FriendsView(vec![
            core_direct_dial::FriendView {
                id: freenet_libp2p_bevy_core::identity::PeerId::from("bob"),
                name: String::from("bob"),
                presence: core_direct_dial::FriendPresence::InRoom(
                    freenet_libp2p_bevy_core::identity::RoomName::from("room-7"),
                ),
            },
            core_direct_dial::FriendView {
                id: freenet_libp2p_bevy_core::identity::PeerId::from("mia"),
                name: String::from("mia"),
                presence: core_direct_dial::FriendPresence::Online,
            },
            core_direct_dial::FriendView {
                id: freenet_libp2p_bevy_core::identity::PeerId::from("adam"),
                name: String::from("12D3Koo...99RC"),
                presence: core_direct_dial::FriendPresence::Offline,
            },
        ])
    }

    fn closed(_world: &mut World) {}

    fn open(world: &mut World) {
        world.insert_resource(ui::direct_dial::FriendsDrawerState(true));
        let mut panels = world.query_filtered::<&mut ui::direct_dial::FriendsDrawerSlide, With<ui::direct_dial::FriendsDrawerPanel>>();
        if let Ok(mut slide) = panels.single_mut(world) {
            **slide = ui::constants::DRAWER_SLIDE_SECS;
        }
    }

    fn open_with_friends(world: &mut World) {
        open(world);
        *world.resource_mut::<core_direct_dial::FriendsView>() = preview_view();
    }

    fn open_with_error(world: &mut World) {
        open_with_friends(world);
        **world.resource_mut::<core_direct_dial::FriendAddressError>() = Some(String::from(
            "bad address: use /ip4/<host>/tcp/<port>/p2p/<id>",
        ));
    }

    fn scene(states: Vec<lele_bevy_preview::scene::State>) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("friends_ui_plugin"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states,
            timeline: None,
        }
    }

    #[test]
    fn friends_ui_plugin_ui_scene_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        scene(vec![
            lele_bevy_preview::scene::State {
                label: String::from("closed"),
                apply: closed,
            },
            lele_bevy_preview::scene::State {
                label: String::from("open empty"),
                apply: open,
            },
            lele_bevy_preview::scene::State {
                label: String::from("open with three friends"),
                apply: open_with_friends,
            },
            lele_bevy_preview::scene::State {
                label: String::from("address error shown"),
                apply: open_with_error,
            },
        ])
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use crate::ui;
    use bevy::prelude::Plugin;
    use bevy::prelude::*;
    use lele_bevy_state_machine::traits::StateMachine;

    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::time;

    fn empty_state() -> core_direct_dial::state_machine::DialState {
        core_direct_dial::state_machine::DialState::from_peer_book(
            &core_direct_dial::PeerBook::default(),
            time::UnixTime::from_secs(10),
        )
    }

    fn app_with_plugins() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_direct_dial::state_machine::Input>();
        app.add_message::<core_direct_dial::FriendAddressSubmitted>();
        app.insert_resource(empty_state());
        app.insert_resource(core_direct_dial::FriendAddressError(None));
        app.init_resource::<core_direct_dial::FriendsView>();
        app.init_resource::<core_direct_dial::OwnShareAddress>();
        Plugin::build(&ui::direct_dial::FriendsUiPlugin, &mut app);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_plugins();
        app.update();
        let mut drawers = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendsDrawer>>();
        assert_eq!(drawers.iter(app.world()).count(), 1);
        let mut states = app
            .world_mut()
            .query_filtered::<&ui::direct_dial::FriendsDrawerState, ()>();
        assert_eq!(states.iter(app.world()).count(), 1);
    }

    #[test]
    fn ui_adds_a_row_through_real_button_and_input() {
        use bevy::text::EditableText;
        use bevy::ui::Pressed;

        let mut app = app_with_plugins();
        app.add_systems(Update, core_direct_dial::bevy_systems::rebuild_friends_view);
        app.update();
        let id = libp2p::PeerId::random();
        let text = format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}");
        {
            let world = app.world_mut();
            let mut inputs = world
                .query_filtered::<&mut EditableText, With<ui::direct_dial::FriendAddressInput>>();
            if let Ok(mut input) = inputs.single_mut(world) {
                *input = EditableText::new(text.clone());
            } else {
                panic!("friend input missing");
            }
        }
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        let submitted: Vec<core_direct_dial::FriendAddressSubmitted> = app
            .world_mut()
            .resource_mut::<Messages<core_direct_dial::FriendAddressSubmitted>>()
            .drain()
            .collect();
        assert_eq!(submitted.len(), 1);
        assert_eq!(**submitted[0], text);
        let peer = identity::Peer::try_from(text.as_str()).expect("peer");
        app.world_mut()
            .resource_mut::<core_direct_dial::state_machine::DialState>()
            .update(
                core_direct_dial::state_machine::Input::FriendAdded(peer),
                time::UnixTime::from_secs(10),
            );
        app.update();
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendRow>>();
        assert_eq!(rows.iter(app.world()).count(), 1);
    }

    #[test]
    fn the_copy_chip_writes_the_full_address_to_the_clipboard() {
        use bevy::ui::Pressed;

        let mut app = app_with_plugins();
        app.update();
        **app
            .world_mut()
            .resource_mut::<core_direct_dial::OwnShareAddress>() =
            Some(core_direct_dial::OwnAddress {
                id: identity::PeerId::from("me"),
                short: String::from("12D3Koo...99RC"),
                full: Some(String::from("/ip4/127.0.0.1/tcp/4001/p2p/me")),
            });
        app.update();
        let chip = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::OwnAddressChip>>()
            .single(app.world())
            .expect("chip");
        app.world_mut().entity_mut(chip).insert(Pressed);
        app.update();
        let mut toasts = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::CopyToast>>();
        assert_eq!(toasts.iter(app.world()).count(), 1);
    }
}
