mod args;
mod friends_drawer;
mod journey;
mod ring;
mod script_runner;

use bevy::prelude::*;
use clap::Parser;
use examples_shared as shared;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::ui as plugin_ui;
use freenet_libp2p_bevy_plugin::testing;
use lobby_room::app::LobbyRoomApp;
use lobby_room::{ping, ui as example_ui};

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
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(LobbyRoomApp::primary_window(&args.username)),
        ..default()
    }));
    let ring = ring::RingPeers::new();
    if matches!(args.network.network, testing::Network::Fake) {
        ring.set(1);
    } else {
        ring::spawn_ring_poller(ws_port, ring.clone());
    }
    app.insert_resource(ring);
    app.insert_resource(shared::intent::Intent::new(None));
    app.insert_resource(testing::PendingAction::default());
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
    let fake = testing::FakeNetworkPlugin::<ping::PingPayload>::from(&args.network);
    let lobby = freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyFakeNetworkPlugin::new(&fake);
    app.add_plugins((fake, lobby));
    app.add_plugins(LobbyRoomApp {
        username: identity::Username::from(args.username.clone()),
        game,
        transport: args::transport_mode(args.transport),
        peer_book_file: args.peer_book_file.clone(),
        freenet_port: ws_port,
        room_ping: args.is_room_ping_enabled,
        ui: args::ui_choice(args.ui),
    });
    // scripted input runs through Bevy's real typing/clicking paths
    match args.ui {
        args::UiArg::Default => {
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
            app.add_systems(
                Update,
                shared::intent::script_input::<
                    example_ui::NameField,
                    example_ui::NewRoomButton,
                    example_ui::JoinButton,
                >,
            );
        }
    }
    app.add_systems(
        Update,
        (
            shared::intent::log_joined,
            shared::status::log_tick,
            shared::status::log_mesh,
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
