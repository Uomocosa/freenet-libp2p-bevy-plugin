use crate::ui;
use bevy::prelude::*;

use ui::discovery::SelectedRoom;

pub fn reset_room_selection(mut selection: ResMut<SelectedRoom>) {
    *selection = SelectedRoom::default();
}
