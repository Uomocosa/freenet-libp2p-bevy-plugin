use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::WindowPosition;
use bevy::winit::{UpdateMode, WinitSettings};
use clap::Parser;
use e2e_apps::transport_arg::TransportArg;
use e2e_apps::window_origin::WindowOrigin;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::testing::{LobbyE2EMethods, LobbyFakeNetworkPlugin};
use freenet_libp2p_bevy_plugin::testing::{self, NetworkArgs};
use lobby_room::app::LobbyRoomApp;
use lobby_room::ping::PingPayload;

use crate::ui_arg::UiArg;
use e2e_apps::lobby_room::LobbyRoomE2EMethods;

#[derive(Debug, Parser)]
#[command(name = "lobby_room_e2e")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    #[arg(long, value_enum, default_value = "default")]
    pub ui: UiArg,
    #[arg(long)]
    pub room_ping: bool,
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
        let mut window = LobbyRoomApp::primary_window(&self.username);
        if let Some(origin) = self.window_position {
            window.position = WindowPosition::At(origin.0);
        }
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        });
        let fake = testing::FakeNetworkPlugin::<PingPayload>::from(&self.network);
        let lobby = LobbyFakeNetworkPlugin::new(&fake);
        app.add_plugins((fake, lobby));
        app.add_plugins(LobbyRoomApp {
            username: identity::Username::from(self.username.clone()),
            game: game.clone(),
            transport: self.transport.mode(),
            peer_book_file: None,
            freenet_port: ws_port,
            room_ping: self.room_ping,
            ui: self.ui.choice(),
        });
        app.add_plugins(
            testing::E2EDriverPlugin::new(self.e2e_port)
                .with(LobbyE2EMethods)
                .with(LobbyRoomE2EMethods),
        );
        app.run()
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Args;

    #[test]
    fn test_usage() {
        let args = Args::try_parse_from([
            "lobby_room_e2e",
            "--username",
            "alice",
            "--e2e-port",
            "15710",
            "--room-ping",
        ])
        .expect("args");
        assert!(args.room_ping);
        assert_eq!(args.e2e_port, 15710);
    }
}
