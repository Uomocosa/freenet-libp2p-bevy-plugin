use crate::ui;
use std::f32::consts::TAU;

use bevy::prelude::*;

use ui::discovery::SPIN_TURNS_PER_SEC;
use ui::discovery::SpinnerRing;

pub fn spin_spinners(time: Res<Time>, mut rings: Query<&mut UiTransform, With<SpinnerRing>>) {
    let angle = time.into_inner().elapsed_secs() * SPIN_TURNS_PER_SEC * TAU;
    for mut transform in &mut rings {
        transform.rotation = Rot2::radians(angle);
    }
}
