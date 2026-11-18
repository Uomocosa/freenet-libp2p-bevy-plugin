use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::CreateShake;
use ui::discovery::{CREATE_PRESS_SCALE, CREATE_SHAKE_PX, CREATE_SHAKE_SECS, CREATE_SHAKE_TURNS};

type CreateParts<'a> = (
    Entity,
    Option<&'a mut CreateShake>,
    Has<Pressed>,
    &'a mut UiTransform,
);

pub fn animate_create_press(
    time: Res<Time>,
    mut buttons: Query<CreateParts, With<ui::discovery::CreateRoomButton>>,
    mut commands: Commands,
) {
    let delta = time.into_inner().delta_secs();
    for (entity, shake, pressed, mut transform) in &mut buttons {
        let scale = if pressed { CREATE_PRESS_SCALE } else { 1.0 };
        transform.scale = Vec2::splat(scale);
        transform.translation.x = Val::ZERO;
        let Some(mut shake) = shake else {
            continue;
        };
        let elapsed = (**shake + delta).min(CREATE_SHAKE_SECS);
        **shake = elapsed;
        let progress = elapsed / CREATE_SHAKE_SECS;
        let offset = CREATE_SHAKE_PX
            * (1.0 - progress)
            * (progress * CREATE_SHAKE_TURNS * std::f32::consts::TAU).sin();
        transform.translation.x = Val::Px(offset);
        if elapsed >= CREATE_SHAKE_SECS {
            commands.entity(entity).remove::<CreateShake>();
        }
    }
}
