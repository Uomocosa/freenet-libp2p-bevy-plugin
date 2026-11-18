use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::EscMenuFriendsButton;

pub fn start_friends_beat(
    presses: Query<(), (Added<Pressed>, With<EscMenuFriendsButton>)>,
    mut beat: ResMut<ui::discovery::FriendsPressBeat>,
) {
    if presses.is_empty() {
        return;
    }
    **beat = ui::discovery::MENU_FRIENDS_BEAT_SECS;
}
