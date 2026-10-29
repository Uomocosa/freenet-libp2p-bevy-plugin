use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

use crate::script::StarboardScriptStep;
use crate::shared::script::timed_script_step::{TimedScriptStep, parse_timed_script_step};

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
    /// The scripted session, a comma-separated list of `verb[:arg][@Ns]` steps.
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_timed_script_step::<StarboardScriptStep>
    )]
    pub script: Option<Vec<TimedScriptStep<StarboardScriptStep>>>,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    /// Folder the embedded Freenet node keeps its config, data and logs in (default: a temp dir).
    #[arg(long)]
    pub freenet_node_dir: Option<PathBuf>,
    /// Where to write this peer's result.json on exit (default: not written).
    #[arg(long)]
    pub result_json: Option<PathBuf>,
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
    use super::{StarboardScriptStep, TransportArg, transport_mode};
    use crate::shared::script::shared_script_step::SharedScriptStep;
    use clap::Parser;
    use freenet_libp2p_bevy_plugin::identity;
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
        let steps = args.script.expect("script");
        assert_eq!(steps.len(), 4);
        assert_eq!(
            steps.first().map(|timed| &timed.step),
            Some(&StarboardScriptStep::Shared(SharedScriptStep::CreateRoom(
                identity::RoomName::from("room-1")
            )))
        );
    }

    #[test]
    fn a_bad_step_is_rejected_with_the_reason() {
        let error = super::Args::try_parse_from([
            "starboard",
            "--username",
            "p1",
            "--script",
            "paint-from-to:10.20-9999.5",
        ])
        .expect_err("bad stroke");
        assert!(error.to_string().contains("outside the canvas"));
    }
}
