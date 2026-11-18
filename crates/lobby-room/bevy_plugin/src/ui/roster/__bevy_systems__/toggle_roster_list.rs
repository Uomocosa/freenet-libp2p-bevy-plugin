use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::roster::{ListState, RosterPill};

pub fn toggle_roster_list(
    presses: Query<(), (Added<Pressed>, With<RosterPill>)>,
    mut list: ResMut<ListState>,
) {
    if presses.is_empty() {
        return;
    }
    list.toggle();
}
