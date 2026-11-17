#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::enums::{BootstrapOutcome, SpectateFreshness};
pub use basic::resources::{
    Spectated, SpectatorCount, SpectatorEvents, SpectatorGame, SpectatorInputs, SpectatorMachine,
};

mod history_bootstrap;
pub use history_bootstrap::HistoryBootstrap;

mod plugin;
pub use plugin::SpectatorPlugin;

pub mod state_machine;
pub use state_machine::SpectateCommand;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
