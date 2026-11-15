mod args;
mod friends_drawer;
mod journey;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::freenet_node;
use freenet_libp2p_bevy_plugin::identity;
use lobby_room::app::LobbyRoomApp;
use lobby_room::ring;

#[tokio::main]
async fn main() -> bevy::app::AppExit {
    let args = args::Args::parse();
    init_tracing();
    let node = match freenet_node::EmbeddedNode::start(args.freenet_node_dir.clone()).await {
        Ok(node) => node,
        Err(error) => {
            eprintln!("failed to start embedded freenet node: {error}");
            return bevy::app::AppExit::error();
        }
    };
    let ws_port = node.websocket_port;
    run_lobby(&args, ws_port)
}

fn run_lobby(args: &args::Args, ws_port: u16) -> bevy::app::AppExit {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(LobbyRoomApp::primary_window(&args.username)),
        ..default()
    }));
    let ring = ring::RingPeers::new();
    ring::spawn_ring_poller(ws_port, ring.clone());
    app.insert_resource(ring);
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    app.add_plugins(LobbyRoomApp {
        username: identity::Username::from(args.username.clone()),
        game,
        transport: args.transport,
        peer_book_file: args.peer_book_file.clone(),
        freenet_port: ws_port,
        room_ping: args.is_room_ping_enabled,
        ui: args.ui,
    });
    app.run()
}

// needed helper: stdout tracing so the run log captures every line
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .init();
}
