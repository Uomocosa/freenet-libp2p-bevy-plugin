#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::constants::{
    LEASE_INTERVAL_PER_SPECTATOR, LEASE_TTL_MULTIPLIER, MAX_LEASE_INTERVAL,
    MAX_SLICE_REQUESTS_PER_PEER, MIN_LEASE_INTERVAL, SLICE_REQUEST_COOLDOWN,
};
pub use basic::enums::{SpectateInput, SpectateOutput, SpectatePhase};
pub use basic::messages::SpectateCommand;

mod spectator_state;
pub use spectator_state::SpectatorState;
