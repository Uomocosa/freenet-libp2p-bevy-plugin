use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::{Res, ResMut};

use crate::testing;

pub fn count_escape_presses(
    keys: Res<ButtonInput<KeyCode>>,
    presses: ResMut<testing::EscapePresses>,
) {
    let keys = keys.into_inner();
    let presses = presses.into_inner();
    if keys.just_pressed(KeyCode::Escape) {
        let total = **presses;
        **presses = total.saturating_add(1);
    }
}
