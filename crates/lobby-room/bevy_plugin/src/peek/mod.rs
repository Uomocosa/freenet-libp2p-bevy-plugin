#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::constants::{
    PREVIEW_STOP_LINGER_SECS, PREVIEW_TARGET_HEIGHT, PREVIEW_TARGET_WIDTH, STREAM_TARGET_HEIGHT,
    STREAM_TARGET_WIDTH,
};
pub use basic::resources::{RoomPeekGame, RoomPreviewTarget, RoomStreamTarget};

mod room_peek_plugin;
pub use room_peek_plugin::RoomPeekPlugin;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
