#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{RollMachine, RollNumber, RollNumberColumn, RollNumberRow};
pub use basic::constants::{REVEAL_SECS, ROLL_SECS};
pub use basic::enums::{Input, Output, Phase, RollSet};

mod number_style;
pub use number_style::NumberStyle;

mod state;
pub use state::State;

mod ui_plugin;
pub use ui_plugin::UiPlugin;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
