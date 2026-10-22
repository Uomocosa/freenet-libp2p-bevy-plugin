use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

use crate::shared::script::shared_script_step::SharedScriptStep;
use crate::shared::script::timed_script_step::{TimedScriptStep, parse_timed_script_step};

/// Which lobby UI to run: the plugin's default one or the example's replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum UiArg {
    Default,
    Custom,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TransportArg {
    Tcp,
    Quic,
    Both,
}

#[derive(Debug, Parser)]
#[command(name = "lobby_room")]
pub struct Args {
    #[arg(long)]
    pub username: String,
    /// The scripted session, a comma-separated list of `verb[:arg][@Ns]` steps.
    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_timed_script_step::<SharedScriptStep>
    )]
    pub script: Option<Vec<TimedScriptStep<SharedScriptStep>>>,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
    #[arg(long, value_enum, default_value = "default")]
    pub ui: UiArg,
    #[arg(long = "room-ping")]
    pub is_room_ping_enabled: bool,
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
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;

    #[test]
    fn test_usage() {
        assert_eq!(transport_mode(TransportArg::Tcp), TransportMode::Tcp);
        assert_eq!(transport_mode(TransportArg::Quic), TransportMode::Quic);
        assert_eq!(transport_mode(TransportArg::Both), TransportMode::Both);
    }
}
