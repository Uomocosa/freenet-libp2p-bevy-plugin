#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
pub mod state_machine;

#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::constants::*;
pub use basic::enums::{
    HowRoomWasEntered, JoinPhase, MemberRoomStatus, OwnRoomStatus, RoomOccupancy,
};
pub use basic::messages::{Command, Event};
pub use basic::resources::{
    FreenetPeersReceiver, FreenetPort, HeartbeatTargetSender, Libp2pEventTap, LobbyRows,
    LobbyUpdateReceiver, RoomRoster,
};
pub use basic::structs::{JoinedRoom, LobbyRow, LobbyTaskChannels, RoomRosterEntry};
pub use basic::type_aliases::Members;
pub use common_types::{Lobby, Room, RoomMember, RoomMembership, RoomSize, RoomSizeError};

mod lobby_contract_subscription;
pub use lobby_contract_subscription::LobbyContractSubscription;

mod lobby_lease_limits;
pub use lobby_lease_limits::LobbyLeaseLimits;

mod lobby_task_config;
pub use lobby_task_config::LobbyTaskConfig;

mod resubscribe_gate;
pub use resubscribe_gate::ResubscribeGate;

#[cfg(feature = "testing")]
mod lobby_task_override;
#[cfg(feature = "testing")]
pub use lobby_task_override::LobbyTaskOverride;

mod config;
pub use config::Config;

mod holder_heartbeat_intervals;
pub use holder_heartbeat_intervals::HolderHeartbeatIntervals;

mod history_size_limits;
pub use history_size_limits::HistorySizeLimits;

mod error;
pub use error::Error;

mod game_id;

mod plugin;
pub use plugin::Plugin;

mod plugins;
pub use plugins::Plugins;
