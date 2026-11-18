use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{JoinError, JoinErrorDismissButton};

pub fn dismiss_join_failed(
    buttons: Query<(), (Added<Pressed>, With<JoinErrorDismissButton>)>,
    mut join_error: ResMut<JoinError>,
) {
    if buttons.iter().next().is_some() {
        **join_error = None;
    }
}
