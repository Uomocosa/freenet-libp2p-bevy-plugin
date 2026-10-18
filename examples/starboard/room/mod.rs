#[path = "__basic__/mod.rs"]
pub mod basic;
mod is_in_room;

pub mod bevy_systems;

pub use basic::constants::BRUSH_REFRESH_SECONDS;
pub use basic::resources::{BrushRefreshTimer, RoomBrushes};
pub use is_in_room::is_in_room;
