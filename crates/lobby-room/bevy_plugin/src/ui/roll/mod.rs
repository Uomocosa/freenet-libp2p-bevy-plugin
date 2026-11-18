#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{RollDigit, RollMachine, RollNumber};
pub use basic::constants::{DIGIT_ROWS, REVEAL_SECS, ROLL_SECS};
pub use basic::enums::{Input, Output, Phase};
pub use basic::structs::RollNumberStyle;

mod state;
pub use state::State;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
