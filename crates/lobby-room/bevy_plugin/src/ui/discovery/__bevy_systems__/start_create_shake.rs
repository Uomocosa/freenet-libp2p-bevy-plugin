use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::CreateShake;

pub fn start_create_shake(
    pressed: Query<Entity, (Added<Pressed>, With<ui::discovery::CreateRoomButton>)>,
    mut commands: Commands,
) {
    for entity in &pressed {
        commands.entity(entity).insert(CreateShake::default());
    }
}
