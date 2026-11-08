pub mod bevy_systems;
pub mod state_machine;
#[cfg(feature = "default_ui")]
pub mod ui;

#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::constants::*;
pub use basic::enums::{HowRoomWasEntered, JoinPhase, RoomMembership};
pub use basic::messages::{Command, Event};
pub use basic::resources::{CommandSender, EventReceiver, FreenetPort, StatusReceiver};
pub use basic::structs::{
    JoinedRoom, LobbyLeaseLimits, LobbyTaskChannels, RoomMember, RunChannels,
};
pub use basic::type_aliases::Members;

mod lobby;
pub use lobby::Lobby;

mod lobby_contract_subscription;
pub use lobby_contract_subscription::LobbyContractSubscription;

mod lobby_task_config;
pub use lobby_task_config::LobbyTaskConfig;

#[cfg(feature = "testing")]
mod lobby_task_override;
#[cfg(feature = "testing")]
pub use lobby_task_override::LobbyTaskOverride;

mod lobby_room;
pub use lobby_room::LobbyRoom;

mod room_size;
pub use room_size::RoomSize;

mod status;
pub use status::Status;

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
