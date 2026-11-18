use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::direct_dial::{FriendsDrawerBackdrop, FriendsDrawerCloseButton, FriendsDrawerState};

type DrawerPresses = Or<(With<FriendsDrawerBackdrop>, With<FriendsDrawerCloseButton>)>;

pub fn close_drawer_on_press(
    presses: Query<(), (Added<Pressed>, DrawerPresses)>,
    mut state: ResMut<FriendsDrawerState>,
) {
    if presses.is_empty() {
        return;
    }
    state.close();
}
