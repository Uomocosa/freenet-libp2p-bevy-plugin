use bevy::prelude::Component;
use derive_more::Deref;

use crate::net_id;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct UiRoot;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomNameInput;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CreateRoomButton;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RoomList;

#[derive(Component, Debug, Clone, PartialEq, Eq, Deref)]
pub struct RoomButton(pub net_id::RoomName);
