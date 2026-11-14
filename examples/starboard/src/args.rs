use std::path::{Path, PathBuf};

use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use examples_shared as shared;
use examples_shared::e2e_report::peer_run_result::PeerRunResult;
use freenet_libp2p_bevy_plugin::lobby_room::ui;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;
use freenet_libp2p_bevy_plugin::testing::{E2EArgs, NetworkArgs};
use freenet_libp2p_bevy_plugin::{identity, testing};

use crate::scripted_starboard_step::ScriptedStarboardStep;
use crate::{friend_intent, result, scaled_timings, script_runner, starboard, status, time_scale};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TransportArg {
    Tcp,
    Quic,
    Both,
}

#[derive(Debug, Parser)]
#[command(name = "starboard")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    /// Folder the embedded Freenet node keeps its config, data and logs in (default: a temp dir).
    #[arg(long)]
    pub freenet_node_dir: Option<PathBuf>,
    #[arg(long)]
    pub identity_file: Option<PathBuf>,
    #[arg(long)]
    pub peer_book_file: Option<PathBuf>,
    #[command(flatten)]
    pub network: NetworkArgs,
    #[command(flatten)]
    pub e2e: E2EArgs<ScriptedStarboardStep>,
}

impl Args {
    pub(crate) fn run_starboard(
        &self,
        ws_port: u16,
        game: &identity::GameId,
        time_scale: time_scale::Scale,
    ) -> bevy::app::AppExit {
        let keypair = match self.identity_file.as_deref().map(Self::load_keypair) {
            Some(Ok(saved)) => Some(saved),
            Some(Err(error)) => {
                eprintln!("{error}");
                return bevy::app::AppExit::error();
            }
            None => None,
        };
        let game_app = starboard::app::StarboardApp {
            username: identity::Username::from(self.username.clone()),
            game: game.clone(),
            transport: transport_mode(self.transport),
            keypair,
            peer_book_file: self.peer_book_file.clone(),
            freenet_port: ws_port,
            lease: scaled_timings::scaled_lease_timings(time_scale),
            history: scaled_timings::scaled_history_config(time_scale),
        };
        let mut app = App::new();
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(starboard::app::StarboardApp::primary_window(&self.username)),
            ..default()
        }));
        app.insert_resource(shared::status::LocalUser(identity::Username::from(
            self.username.clone(),
        )));
        app.insert_resource(shared::intent::Intent::new(None));
        app.insert_resource(testing::PendingAction::default());
        app.insert_resource(friend_intent::FriendIntent::default());
        app.add_systems(Update, friend_intent::drive_friend_intent);
        let fake = testing::FakeNetworkPlugin::<starboard::room::CursorUpdate>::from(&self.network);
        let lobby =
            freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyFakeNetworkPlugin::new(&fake);
        app.add_plugins((fake, lobby));
        app.add_plugins(game_app);
        app.add_systems(
            Update,
            shared::intent::script_input::<
                ui::discovery::RoomNameInput,
                ui::discovery::CreateRoomButton,
                ui::discovery::RoomButton,
            >,
        );
        if let Some(steps) = match self.e2e.parsed_script() {
            Ok(steps) => steps,
            Err(error) => {
                eprintln!("{error}");
                return bevy::app::AppExit::error();
            }
        } {
            app.insert_resource(script_runner::StarboardScriptRunner::new(steps));
            app.add_systems(
                Update,
                testing::bevy_systems::run_script::<ScriptedStarboardStep>.before(
                    shared::intent::script_input::<
                        ui::discovery::RoomNameInput,
                        ui::discovery::CreateRoomButton,
                        ui::discovery::RoomButton,
                    >,
                ),
            );
        }
        app.add_systems(
            Update,
            (
                shared::intent::log_joined,
                shared::status::log_tick,
                shared::status::log_mesh,
                status::log_canvas_hash,
            ),
        );
        if let Some(path) = self.e2e.result_json.clone() {
            self.record_result(&mut app, path);
        }
        app.run()
    }

    pub(crate) fn load_keypair(path: &Path) -> Result<identity::SavedKeypair, String> {
        identity::SavedKeypair::try_from(path).map_err(|error| format!("identity file: {error}"))
    }

    pub(crate) fn record_result(&self, app: &mut App, path: PathBuf) {
        app.insert_resource(PeerRunResult {
            username: self.username.clone(),
            ..PeerRunResult::default()
        });
        app.insert_resource(result::ResultJsonPath(path));
        app.add_systems(
            Update,
            result::bevy_systems::record_peer_run_result
                .after(testing::bevy_systems::run_script::<ScriptedStarboardStep>)
                .run_if(resource_exists::<script_runner::StarboardScriptRunner>),
        );
        app.add_systems(Last, result::bevy_systems::write_peer_run_result);
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

#[must_use]
pub const fn transport_mode(arg: TransportArg) -> TransportMode {
    match arg {
        TransportArg::Tcp => TransportMode::Tcp,
        TransportArg::Quic => TransportMode::Quic,
        TransportArg::Both => TransportMode::Both,
    }
}

#[cfg(test)]
mod tests {
    use super::{TransportArg, transport_mode};
    use crate::scripted_starboard_step::ScriptedStarboardStep;
    use clap::Parser;
    use e2e_steps::starboard_steps::StarboardStep;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::lobby_room::testing::LobbyStep;
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;

    #[test]
    fn test_usage() {
        assert_eq!(transport_mode(TransportArg::Tcp), TransportMode::Tcp);
        assert_eq!(transport_mode(TransportArg::Quic), TransportMode::Quic);
        assert_eq!(transport_mode(TransportArg::Both), TransportMode::Both);
    }

    #[test]
    fn a_script_is_a_comma_separated_list_of_steps() {
        let args = super::Args::try_parse_from([
            "starboard",
            "--username",
            "p1",
            "--script",
            "create:room-1, wait-join-succeeded:room-1@60s, paint-from-to:10.20-30.102, exit-app",
        ])
        .expect("args");
        let steps = args.e2e.parsed_script().expect("parsed").expect("script");
        assert_eq!(steps.len(), 4);
        assert_eq!(
            steps.first().map(|timed| &timed.step),
            Some(&ScriptedStarboardStep::from(StarboardStep::Lobby(
                LobbyStep::CreateRoom(identity::RoomName::from("room-1"))
            )))
        );
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
    }

    #[test]
    fn a_bad_step_is_rejected_with_the_reason() {
        let args = super::Args::try_parse_from([
            "starboard",
            "--username",
            "p1",
            "--script",
            "paint-from-to:10.20-9999.5",
        ])
        .expect("args");
        let error = args.e2e.parsed_script().expect_err("bad stroke");
        assert!(error.contains("outside the canvas"));
    }
}
