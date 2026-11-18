#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{FriendHighlight, SlotBoardSprite, SlotCamera};
pub use basic::constants::{
    FRIEND_HIGHLIGHT_BAND_PX, FRIEND_HIGHLIGHT_RGBA, FRIEND_HIGHLIGHT_RING_PX, FRIEND_HIGHLIGHT_Z,
    FRIEND_LABEL_COLOR, PREVIEW_BACKDROP_COLOR, PREVIEW_BOARD_SIDE, PREVIEW_CAMERA_ORDER,
    PREVIEW_LAYER, PREVIEW_SLOT, SLOT_RENDERS, STREAM_BACKDROP_COLOR, STREAM_BOARD_SIDE,
    STREAM_CAMERA_ORDER, STREAM_LAYER,
};
pub use basic::resources::SlotCursors;
pub use basic::structs::SlotRenderConfig;

mod friend_ring;
pub use friend_ring::friend_ring;

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
