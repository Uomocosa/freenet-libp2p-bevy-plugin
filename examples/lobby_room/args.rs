use std::str::FromStr;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::identity::RoomName;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

/// Scripted interaction: `create:<room>` types the room name into the input and
/// presses "Create New Room"; `join:<room>` presses that room's button once it shows
/// up in the list. The name is always explicit — an empty name is rejected.
#[derive(Debug, Clone)]
pub enum Action {
    Create(RoomName),
    Join(RoomName),
}

impl FromStr for Action {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let (kind, room) = raw
            .split_once(':')
            .ok_or_else(|| format!("expected create:<room> or join:<room>, got {raw}"))?;
        if room.trim().is_empty() {
            return Err("room name is empty".to_string());
        }
        let room = RoomName::from(room);
        match kind {
            "create" => Ok(Self::Create(room)),
            "join" => Ok(Self::Join(room)),
            other => Err(format!("expected create or join, got {other}")),
        }
    }
}

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
    #[arg(long)]
    pub action: Option<Action>,
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
    use super::Action;
    use freenet_libp2p_bevy_plugin::identity::RoomName;

    #[test]
    fn test_usage() {
        assert!(matches!(
            "create:alpha".parse::<Action>(),
            Ok(Action::Create(room)) if room == RoomName::from("alpha")
        ));
        assert!(matches!(
            "join:beta".parse::<Action>(),
            Ok(Action::Join(room)) if room == RoomName::from("beta")
        ));
        // a create without an explicit, non-empty name is rejected
        assert!("create".parse::<Action>().is_err());
        assert!("create:".parse::<Action>().is_err());
        assert!("create:   ".parse::<Action>().is_err());
        assert!("frob:alpha".parse::<Action>().is_err());
    }
}
