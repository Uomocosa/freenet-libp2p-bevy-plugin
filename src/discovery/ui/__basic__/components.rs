use bevy::prelude::Component;
use derive_more::{Deref, DerefMut};

use crate::identity;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Root;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomNameInput;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CreateRoomButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomList;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RoomButton(pub identity::RoomName);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct IsCurrentRoom;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveDoorRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveDoor;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct DoorOpening(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DoorPanel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct DoorLabel;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveYesButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct LeaveNoButton;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct HoverGlow(pub f32);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct HoverOutlineOnly;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ConnectingRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SpinnerRing;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoiningRoomRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoinErrorRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct JoinErrorDismissButton;
