use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::WindowPosition;
use bevy::winit::{UpdateMode, WinitSettings};
use clap::Parser;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;
use freenet_libp2p_bevy_plugin::room_sync;
use freenet_libp2p_bevy_plugin::testing::{self, NetworkArgs};
use freenet_libp2p_bevy_plugin::{identity, lobby_room};
use starboard::starboard::app::StarboardApp;
use starboard::starboard::room::CursorUpdate;

use e2e_apps::starboard::StarboardE2EMethods;
use e2e_apps::transport_arg::TransportArg;
use e2e_apps::window_origin::WindowOrigin;

#[derive(Debug, Parser)]
#[command(name = "starboard_e2e")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    #[arg(long)]
    pub freenet_node_dir: Option<PathBuf>,
    #[command(flatten)]
    pub network: NetworkArgs,
    #[arg(long)]
    pub e2e_port: u16,
    #[arg(long)]
    pub window_position: Option<WindowOrigin>,
}

impl Args {
    pub fn init_tracing() {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .init();
    }

    pub fn run(&self, ws_port: u16, game: &identity::GameId) -> AppExit {
        let game_app = StarboardApp {
            username: identity::Username::from(self.username.clone()),
            game: game.clone(),
            transport: self.transport.mode(),
            keypair: None,
            peer_book_file: None,
            freenet_port: ws_port,
            lease: discovery::HolderHeartbeatIntervals {
                interval_between_heartbeats: Duration::from_secs(30),
                ttl: Duration::from_secs(90),
                remote_clock_skew_margin: Duration::from_mins(10),
            },
            history: room_sync::HistoryConfig {
                write_interval: Duration::from_secs(30),
                record_ttl: Duration::from_mins(10),
                ..room_sync::HistoryConfig::default()
            },
        };
        let mut app = App::new();
        let mut window = StarboardApp::primary_window(&self.username);
        if let Some(origin) = self.window_position {
            window.position = WindowPosition::At(origin.0);
        }
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        });
        let fake = testing::FakeNetworkPlugin::<CursorUpdate>::from(&self.network);
        let lobby = lobby_room::testing::LobbyFakeNetworkPlugin::new(&fake);
        app.add_plugins((fake, lobby))
            .add_plugins(game_app)
            .add_plugins(
                testing::E2EDriverPlugin::new(self.e2e_port)
                    .with(lobby_room::testing::LobbyE2EMethods)
                    .with(StarboardE2EMethods),
            );
        app.run()
    }
}
