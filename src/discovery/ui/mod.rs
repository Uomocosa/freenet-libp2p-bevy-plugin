#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    ConnectingRoot, CreateRoomButton, DoorLabel, DoorOpening, DoorPanel, HoverGlow,
    HoverOutlineOnly, IsCurrentRoom, LeaveDoor, LeaveDoorRoot, LeaveNoButton, LeavePrompt,
    LeaveYesButton, RoomButton, RoomList, RoomNameInput, SpinnerRing, UiRoot,
};
pub use basic::states::{LeaveDialog, UiScreen};
pub use basic::structs::RoomDetails;

pub mod bevy_systems;

mod default_ui_plugin;
pub use default_ui_plugin::DefaultUiPlugin;
