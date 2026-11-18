#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::constants::{REVEAL_SECS, ROLL_SECS};
pub use basic::enums::{Input, Output, Phase};

mod state;
pub use state::State;
