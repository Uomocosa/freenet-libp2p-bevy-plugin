use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

use crate::script::{ScriptStep, parse_script_step};
use crate::shared::action::Action;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TransportArg {
    Tcp,
    Quic,
    Both,
}

#[derive(Debug, Parser)]
#[command(name = "starboard_gossip")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    /// The room action, when the peer is not scripted.
    #[arg(long, conflicts_with = "script")]
    pub action: Option<Action>,
    /// The scripted session, a comma-separated list of steps.
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_script_step,
        conflicts_with = "action"
    )]
    pub script: Option<Vec<ScriptStep>>,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    /// How long the peer waits for its room to show up in the lobby. The
    /// mainnet's contract state can take minutes to reach a cold node.
    #[arg(long, default_value_t = 300)]
    pub join_timeout_secs: u64,
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
    use super::{ScriptStep, TransportArg, transport_mode};
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
            "starboard_gossip",
            "--username",
            "p1",
            "--script",
            "create:room-1, wait-secs:20, paint-from-to:10.20-30.102, exit",
        ])
        .expect("args");
        let steps = args.script.expect("script");
        assert_eq!(steps.len(), 4);
        assert_eq!(
            steps.first(),
            Some(&ScriptStep::CreateRoom(identity::RoomName::from("room-1")))
        );
        assert_eq!(steps.last(), Some(&ScriptStep::ExitGame));
    }

    #[test]
    fn a_script_and_an_action_cannot_both_be_passed() {
        let error = super::Args::try_parse_from([
            "starboard_gossip",
            "--username",
            "p1",
            "--action",
            "join:room-1",
            "--script",
            "join:room-1",
        ])
        .expect_err("conflict");
        assert!(error.to_string().contains("cannot be used with"));
    }

    #[test]
    fn a_bad_step_is_rejected_with_the_reason() {
        let error = super::Args::try_parse_from([
            "starboard_gossip",
            "--username",
            "p1",
            "--script",
            "paint-from-to:10.20-9999.5",
        ])
        .expect_err("bad stroke");
        assert!(error.to_string().contains("outside the canvas"));
    }
}
