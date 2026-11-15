use std::path::{Path, PathBuf};
use std::time::Duration;

use bevy::prelude::*;
use clap::Parser;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;
use freenet_libp2p_bevy_plugin::testing::Network;
use freenet_libp2p_bevy_plugin::{identity, room_sync};
use starboard::starboard;

#[derive(Debug, Parser)]
#[command(name = "starboard")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportMode,
    /// Folder the embedded Freenet node keeps its config, data and logs in (default: a temp dir).
    #[arg(long)]
    pub freenet_node_dir: Option<PathBuf>,
    #[arg(long)]
    pub identity_file: Option<PathBuf>,
    #[arg(long)]
    pub peer_book_file: Option<PathBuf>,
    /// Network to join (default: mainnet).
    #[arg(long, value_enum, default_value = "mainnet")]
    pub network: Network,
}

impl Args {
    pub(crate) fn run_starboard(&self, ws_port: u16, game: &identity::GameId) -> AppExit {
        let keypair = match self.identity_file.as_deref().map(Self::load_keypair) {
            Some(Ok(saved)) => Some(saved),
            Some(Err(error)) => {
                eprintln!("{error}");
                return AppExit::error();
            }
            None => None,
        };
        let game_app = starboard::app::StarboardApp {
            username: identity::Username::from(self.username.clone()),
            game: game.clone(),
            transport: self.transport,
            keypair,
            peer_book_file: self.peer_book_file.clone(),
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
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(starboard::app::StarboardApp::primary_window(&self.username)),
            ..default()
        }));
        app.add_plugins(game_app);
        app.run()
    }

    pub(crate) fn load_keypair(path: &Path) -> Result<identity::SavedKeypair, String> {
        identity::SavedKeypair::try_from(path).map_err(|error| format!("identity file: {error}"))
    }

    pub(crate) fn init_tracing() {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::Network;
    use clap::Parser;
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;

    #[test]
    fn transport_parses_to_transport_mode() {
        let args =
            super::Args::try_parse_from(["starboard", "--username", "p1", "--transport", "quic"])
                .expect("args");
        assert_eq!(args.transport, TransportMode::Quic);
    }

    #[test]
    fn identity_file_is_optional_and_parses() {
        use std::path::PathBuf;
        let args = super::Args::try_parse_from(["starboard", "--username", "p1"]).expect("args");
        assert_eq!(args.identity_file, None);
        assert_eq!(args.peer_book_file, None);
        let args = super::Args::try_parse_from([
            "starboard",
            "--username",
            "p1",
            "--identity-file",
            "/tmp/p1.identity",
            "--peer-book-file",
            "/tmp/p1.friends.json",
        ])
        .expect("args");
        assert_eq!(args.identity_file, Some(PathBuf::from("/tmp/p1.identity")));
        assert_eq!(
            args.peer_book_file,
            Some(PathBuf::from("/tmp/p1.friends.json"))
        );
        assert!(matches!(args.network, Network::Mainnet));
    }
}
