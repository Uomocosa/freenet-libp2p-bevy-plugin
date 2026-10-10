use std::str::FromStr;

use clap::{Parser, ValueEnum};
use freenet_libp2p_bevy_plugin::net_id::RoomName;
use freenet_libp2p_bevy_plugin::p2p::TransportMode;

/// Scripted button press: `create` presses "Create room", `join:<room>`
/// presses that room's button once it shows up in the list.
#[derive(Debug, Clone)]
pub enum Action {
    Create,
    Join(RoomName),
}

impl FromStr for Action {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw == "create" {
            return Ok(Self::Create);
        }
        let room = raw
            .strip_prefix("join:")
            .ok_or_else(|| format!("expected create or join:<room>, got {raw}"))?;
        if room.is_empty() {
            return Err("room name is empty".to_string());
        }
        Ok(Self::Join(RoomName(room.to_string())))
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
}

#[must_use]
pub const fn transport_mode(arg: TransportArg) -> TransportMode {
    match arg {
        TransportArg::Tcp => TransportMode::Tcp,
        TransportArg::Quic => TransportMode::Quic,
        TransportArg::Both => TransportMode::Both,
    }
}
