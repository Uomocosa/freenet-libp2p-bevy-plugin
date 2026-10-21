#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::components::{
    ConnectingRoot, CreateRoomButton, DoorLabel, DoorOpening, DoorPanel, HoverGlow,
    HoverOutlineOnly, IsCurrentRoom, JoinErrorDismissButton, JoinErrorRoot, JoiningRoomRoot,
    LeaveDoor, LeaveDoorRoot, LeaveNoButton, LeavePrompt, LeaveYesButton, RoomButton, RoomList,
    RoomNameInput, Root, SpinnerRing,
};
pub use basic::resources::JoinError;
pub use basic::states::{LeaveDialog, Screen};
pub use basic::structs::RoomRow;

pub mod bevy_systems;

mod default_ui_plugin;
pub use default_ui_plugin::DefaultUiPlugin;
