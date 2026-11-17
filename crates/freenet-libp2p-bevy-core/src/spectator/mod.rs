#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::Spectated;
pub use basic::enums::{
    BootstrapOutcome, InZoneInput, InZoneOutput, MachineInput, MachineOutput, SpectateFreshness,
    SpectateSlot,
};
pub use basic::resources::{SpectatorCount, SpectatorEvents, SpectatorGame, SpectatorInputs};
pub use basic::structs::PendingHistory;

mod history_bootstrap;
pub use history_bootstrap::HistoryBootstrap;

mod in_zone;
pub use in_zone::InZone;

mod plugin;
pub use plugin::SpectatorPlugin;

mod machine;
pub use machine::Machine;

pub mod state_machine;
pub use state_machine::SpectateCommand;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
