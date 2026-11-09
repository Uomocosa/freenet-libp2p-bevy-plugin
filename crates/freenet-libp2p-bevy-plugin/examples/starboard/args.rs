use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::p2p::TransportMode;
use freenet_libp2p_bevy_plugin::testing::{E2EArgs, NetworkArgs};

use crate::script::StarboardStep;

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
    pub e2e: E2EArgs<StarboardStep>,
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
    use crate::script::StarboardStep;
    use clap::Parser;
    use freenet_libp2p_bevy_plugin::identity;
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;
    use freenet_libp2p_bevy_plugin::testing::E2EStep;

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
            Some(&StarboardStep::Shared(E2EStep::CreateRoom(
                identity::RoomName::from("room-1")
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
