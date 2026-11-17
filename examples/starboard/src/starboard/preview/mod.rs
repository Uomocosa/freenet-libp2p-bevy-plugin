#[path = "__basic__/mod.rs"]
mod basic;

pub use basic::components::{PreviewBoardSprite, PreviewCamera};
pub use basic::constants::{
    PREVIEW_BACKDROP_COLOR, PREVIEW_BOARD_SIDE, PREVIEW_CAMERA_ORDER, PREVIEW_LAYER,
};

#[path = "__bevy_systems__/mod.rs"]
pub mod bevy_systems;
