mod args;
mod friends_drawer;
mod journey;
mod overlay;
mod ping;
mod ring;
mod script_runner;
mod ui;

use bevy::prelude::*;
use clap::Parser;
use examples_shared as shared;
use freenet_libp2p_bevy_plugin::ui as plugin_ui;
use freenet_libp2p_bevy_plugin::{direct_dial, discovery, identity, p2p, plugin, testing};

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    init_tracing();
    if let Err(error) = args.network.validate() {
        eprintln!("{error}");
        return bevy::app::AppExit::error();
    }
    let Ok((_node_guard, ws_port)) =
        shared::node::start_embedded_freenet_node(args.freenet_node_dir.clone(), &args.network)
            .await
    else {
        eprintln!("failed to start embedded freenet node");
        return bevy::app::AppExit::error();
    };
    run_lobby(&args, ws_port)
}

fn run_lobby(args: &args::Args, ws_port: u16) -> bevy::app::AppExit {
    let transport = args::transport_mode(args.transport);
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("lobby-{}", args.username),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.insert_resource(discovery::FreenetPort(ws_port));
    let ring = ring::RingPeers::new();
    if matches!(args.network.network, testing::Network::Fake) {
        ring.set(1);
    } else {
        ring::spawn_ring_poller(ws_port, ring.clone());
    }
    app.insert_resource(ring);
    let username = identity::Username::from(args.username.clone());
    app.insert_resource(shared::status::LocalUser(username.clone()));
    app.insert_resource(ping::IsRoomPingEnabled(args.is_room_ping_enabled));
    app.insert_resource(shared::intent::Intent::new(None));
    app.insert_resource(testing::PendingAction::default());
    app.init_resource::<ping::RoomMessageSenders>();
    if let Some(steps) = match args.e2e.parsed_script() {
        Ok(steps) => steps,
        Err(error) => {
            eprintln!("{error}");
            return bevy::app::AppExit::error();
        }
    } {
        app.insert_resource(script_runner::LobbyScriptRunner::new(steps));
        app.add_systems(
            Update,
            testing::bevy_systems::run_script::<script_runner::LobbyStep>
                .after(ping::log_room_messages),
        );
    }
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    app.add_plugins(testing::FakeNetworkPlugin::<ping::PingPayload>::from(
        &args.network,
    ));
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<ping::PingPayload>::new(transport, p2p::MdnsMode::Disabled)
            .with_game(game.clone()),
    ));
    app.add_plugins(direct_dial::FriendsPlugin::new(direct_dial::Config::from(
        args.peer_book_file.clone(),
    )));
    let discovery_plugins = discovery::Plugins(discovery::Config {
        display_name: username,
        game,
        ..discovery::Config::default()
    });
    // scripted input runs through Bevy's real typing/clicking paths
    match args.ui {
        args::UiArg::Default => {
            app.add_plugins((
                discovery_plugins,
                plugin_ui::discovery::DefaultUiPlugin,
                plugin_ui::direct_dial::FriendsUiPlugin,
            ));
            app.add_systems(
                Update,
                shared::intent::script_input::<
                    plugin_ui::discovery::RoomNameInput,
                    plugin_ui::discovery::CreateRoomButton,
                    plugin_ui::discovery::RoomButton,
                >,
            );
        }
        args::UiArg::Custom => {
            app.add_plugins(discovery_plugins);
            app.add_plugins(ui::LobbyUi);
            app.add_systems(
                Update,
                shared::intent::script_input::<ui::NameField, ui::NewRoomButton, ui::JoinButton>,
            );
        }
    }
    app.add_plugins(overlay::Overlay);
    app.add_systems(
        Update,
        (
            shared::intent::log_joined,
            shared::status::log_tick,
            shared::status::log_mesh,
            ping::publish_room_ping,
            ping::log_room_messages,
        ),
    );
    app.run()
}

// needed helper: stdout tracing so the run log captures every marker line
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .init();
}
