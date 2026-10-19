use std::str::FromStr;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

use crate::shared::action::Action;

/// Scripted board interaction for the e2e: `paint:<n>` draws n strokes;
/// `erase:<n>` erases those same cells once the board has content.
#[derive(Debug, Clone, Copy)]
pub enum ScriptArg {
    Paint(u32),
    Erase(u32),
}

impl ScriptArg {
    #[must_use]
    pub const fn strokes(self) -> u32 {
        match self {
            Self::Paint(count) | Self::Erase(count) => count,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Paint(_) => "paint",
            Self::Erase(_) => "erase",
        }
    }
}

impl FromStr for ScriptArg {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (kind, count) = raw
            .split_once(':')
            .ok_or_else(|| format!("expected paint:<n> or erase:<n>, got {raw}"))?;
        let count: u32 = count
            .parse()
            .map_err(|e| format!("stroke count is not a number: {e}"))?;
        if count == 0 {
            return Err("stroke count must be at least 1".to_string());
        }
        match kind {
            "paint" => Ok(Self::Paint(count)),
            "erase" => Ok(Self::Erase(count)),
            other => Err(format!("expected paint or erase, got {other}")),
        }
    }
}

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
    #[arg(long)]
    pub action: Option<Action>,
    #[arg(long)]
    pub script: Option<ScriptArg>,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, value_enum, default_value = "both")]
    pub transport: TransportArg,
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
    use super::{ScriptArg, TransportArg, transport_mode};
    use freenet_libp2p_bevy_plugin::p2p::TransportMode;

    #[test]
    fn test_usage() {
        assert_eq!(transport_mode(TransportArg::Tcp), TransportMode::Tcp);
        assert_eq!(transport_mode(TransportArg::Quic), TransportMode::Quic);
        assert_eq!(transport_mode(TransportArg::Both), TransportMode::Both);

        assert!(matches!(
            "paint:6".parse::<ScriptArg>(),
            Ok(ScriptArg::Paint(6))
        ));
        assert!(matches!(
            "erase:3".parse::<ScriptArg>(),
            Ok(ScriptArg::Erase(3))
        ));
        assert_eq!(
            "erase:3".parse::<ScriptArg>().map(|arg| arg.strokes()),
            Ok(3)
        );
        // a script without a positive count is rejected
        assert!("paint".parse::<ScriptArg>().is_err());
        assert!("paint:0".parse::<ScriptArg>().is_err());
        assert!("paint:many".parse::<ScriptArg>().is_err());
        assert!("frob:3".parse::<ScriptArg>().is_err());
    }
}
