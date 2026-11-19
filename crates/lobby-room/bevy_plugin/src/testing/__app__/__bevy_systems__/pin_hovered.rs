use bevy::picking::hover::Hovered;
use bevy::prelude::{Commands, Entity, Query, With};

use crate::testing;

pub fn pin_hovered(mut commands: Commands, pinned: Query<Entity, With<testing::PinHovered>>) {
    for entity in &pinned {
        commands.entity(entity).insert(Hovered(true));
    }
}
