#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    CreateRoomButton, IsCurrentRoom, RoomButton, RoomList, RoomNameInput, UiRoot,
};
pub use basic::structs::RoomDetails;

pub mod bevy_systems;

mod default_ui_plugin;
pub use default_ui_plugin::DefaultUiPlugin;
