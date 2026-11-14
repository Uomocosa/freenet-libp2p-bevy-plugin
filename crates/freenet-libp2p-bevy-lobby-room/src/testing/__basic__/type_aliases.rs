use bevy::ecs::query::Has;
use bevy::ui::{ComputedNode, InteractionDisabled, UiGlobalTransform};

use crate::ui;

pub type LocatableNode = (
    &'static ComputedNode,
    &'static UiGlobalTransform,
    Has<InteractionDisabled>,
    Has<ui::discovery::RoomNameInput>,
    Has<ui::discovery::CreateRoomButton>,
    Option<&'static ui::discovery::RoomButton>,
);

// no test_usage necessary
