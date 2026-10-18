#[path = "__basic__/mod.rs"]
pub mod basic;
mod is_in_room;

pub mod bevy_systems;

pub use basic::constants::{BRUSH_REFRESH_SECONDS, CURSOR_UPDATE_SECONDS};
pub use basic::resources::{BrushRefreshTimer, RemoteCursors, RoomBrushes};
pub use basic::structs::CursorUpdate;
pub use is_in_room::is_in_room;
