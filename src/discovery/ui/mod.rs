#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    CreateRoomButton, DoorLabel, DoorOpening, DoorPanel, HoverGlow, HoverOutlineOnly,
    IsCurrentRoom, LeaveDoor, LeaveDoorRoot, LeaveNoButton, LeavePrompt, LeaveYesButton,
    RoomButton, RoomList, RoomNameInput, UiRoot,
};
pub use basic::resources::LeavePromptOpen;
pub use basic::structs::RoomDetails;

pub mod bevy_systems;

mod default_ui_plugin;
pub use default_ui_plugin::DefaultUiPlugin;
