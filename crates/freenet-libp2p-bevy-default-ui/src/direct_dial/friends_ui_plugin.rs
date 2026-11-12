use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy)]
pub struct FriendsUiPlugin;

#[rustfmt::skip]
impl bevy::prelude::Plugin for FriendsUiPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}

#[atomic_delegates]
impl FriendsUiPlugin {
    pub fn build_plugin(self, app: &mut App) {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::FriendsUiPlugin;
    use crate::basic::constants::{DRAWER_SLIDE_SECS, ROOM_COLOR};
    use crate::direct_dial;
    use crate::discovery;
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
            .query_filtered::<(), With<direct_dial::FriendsDrawer>>();
        assert_eq!(drawers.iter(app.world()).count(), 1);
        let mut states = app
            .world_mut()
            .query_filtered::<&direct_dial::FriendsDrawerState, ()>();
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
            BackgroundColor(ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(discovery::PANEL_COLOR),
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
                presence: core_direct_dial::FriendPresence::InLobby,
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
        world.insert_resource(direct_dial::FriendsDrawerState(true));
        let mut panels = world.query_filtered::<&mut direct_dial::FriendsDrawerSlide, With<direct_dial::FriendsDrawerPanel>>();
        if let Ok(mut slide) = panels.single_mut(world) {
            **slide = DRAWER_SLIDE_SECS;
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
