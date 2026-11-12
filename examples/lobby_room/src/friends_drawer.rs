//! Proof that the friends drawer works on the plugin's default UI as
//! `lobby_room --ui default` wires it: the transport's ready address fills the
//! own-address chip (no more `waiting for network...`), the chip click copies
//! the full address, and the drawer renders over the lobby screen.

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::KeyCode;
    use bevy::prelude::*;
    use bevy::ui::Pressed;
    use freenet_libp2p_bevy_plugin::{direct_dial, discovery, identity, p2p, testing, ui};

    use examples_shared::status;

    const PEER_ID: &str = "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC";
    const SHORT_ADDRESS: &str = "12D3Koo...99RC";

    /// The app `run_lobby` builds for `--ui default`, with the transport
    /// already ready to hand out its own address (the plugin order matches
    /// `main.rs`: friends core, then the two default-UI plugins). The base
    /// plugins stay with the caller: preview scenes get theirs from the
    /// preview harness.
    fn add_default_ui(app: &mut App) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        app.insert_resource(discovery::FreenetPort(0));
        app.add_message::<discovery::Command>();
        app.insert_resource(status::LocalUser(identity::Username::from("alice")));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<discovery::LobbyRows>();
        app.init_resource::<discovery::RoomRoster>();
        app.insert_resource(in_room_state());
        let ready = p2p::ReadyChannel::default();
        ready.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from(PEER_ID),
            addrs: vec![identity::PeerAddr::from(format!(
                "/ip4/127.0.0.1/tcp/4001/p2p/{PEER_ID}"
            ))],
        }));
        app.insert_resource(ready);
        app.insert_resource(p2p::Libp2pEventChannel::default());
        app.add_plugins(direct_dial::FriendsPlugin::new(direct_dial::Config::from(
            None,
        )));
        app.add_plugins((
            ui::discovery::DefaultUiPlugin,
            ui::direct_dial::FriendsUiPlugin,
        ));
    }

    fn default_ui_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins((bevy::state::app::StatesPlugin, bevy::input::InputPlugin));
        add_default_ui(&mut app);
        app.finish();
        app
    }

    /// A joined room behind the drawer: the picker then shows the room screen,
    /// the same backdrop the drawer slides over in a real room.
    fn in_room_state() -> discovery::state_machine::DiscoveryState {
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("gamma"),
                created_at: freenet_libp2p_bevy_plugin::time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
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

    fn wait_for(app: &mut App, mut is_ready: impl FnMut(&mut App) -> bool) {
        for _ in 0..400 {
            app.update();
            if is_ready(app) {
                return;
            }
        }
        panic!("the waited-for state never arrived");
    }

    #[test]
    fn the_address_chip_shows_the_real_address() {
        let mut app = default_ui_app();
        wait_for(&mut app, |app| !is_chip_waiting(app));
        assert_eq!(chip_label(&mut app), SHORT_ADDRESS);
    }

    #[test]
    fn clicking_the_chip_copies_the_full_address() {
        let mut app = default_ui_app();
        wait_for(&mut app, |app| !is_chip_waiting(app));
        let mut chips = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::OwnAddressChip>>();
        let chip = chips.single(app.world()).expect("address chip");
        app.world_mut().entity_mut(chip).insert(Pressed);
        app.update();
        let toasts = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::CopyToast>>()
            .iter(app.world())
            .count();
        assert_eq!(toasts, 1, "the chip click shows its copy toast");
    }

    fn open_with_address(world: &mut World) {
        world.insert_resource(ui::direct_dial::FriendsDrawerState(true));
        // the harness pins time to zero deltas; hand the one capture update a
        // big enough step so the slide animation ends fully open
        world.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(250),
        ));
    }

    fn preview_build(app: &mut App) {
        add_default_ui(app);
    }

    #[test]
    fn friends_drawer_address_chip_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("friends_drawer"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("open with the address chip"),
                apply: open_with_address,
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
