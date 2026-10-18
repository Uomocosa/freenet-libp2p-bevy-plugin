#[path = "../starboard/mod.rs"]
mod starboard;

#[path = "../shared/mod.rs"]
mod shared;

mod args;
mod status;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, plugin, room_sync};

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    init_tracing();
    let Ok((_node_guard, ws_port)) = shared::node::bootstrap_node().await else {
        eprintln!("failed to start embedded freenet node");
        return;
    };
    let game = args.token.clone().map_or_else(
        || freenet_libp2p_bevy_plugin::game_id!(),
        identity::GameId::from,
    );
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: format!("starboard-{}", args.username),
            resolution: (960, 540).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_systems(Startup, spawn_camera);
    app.insert_resource(discovery::FreenetPort(ws_port));
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<starboard::room::CursorUpdate>::new(
            args::transport_mode(args.transport),
            p2p::MdnsMode::Disabled,
        )
        .with_game(game.clone()),
    ));
    app.add_plugins(discovery::Plugins(discovery::Config {
        display_name: identity::Username::from(args.username.clone()),
        game: game.clone(),
        ..discovery::Config::default()
    }));
    app.add_plugins(room_sync::SyncPlugin::<starboard::canvas::Canvas>::new(
        game,
    ));
    app.add_plugins(starboard::app::StarboardPlugin);
    app.add_systems(Update, status::log_canvas_hash);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
