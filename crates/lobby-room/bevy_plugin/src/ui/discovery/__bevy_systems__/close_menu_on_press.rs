use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{EscMenuBackdrop, EscMenuResumeButton, EscMenuState};

type MenuDismissPresses = Or<(With<EscMenuBackdrop>, With<EscMenuResumeButton>)>;

pub fn close_menu_on_press(
    presses: Query<(), (Added<Pressed>, MenuDismissPresses)>,
    mut menu: ResMut<EscMenuState>,
) {
    if presses.is_empty() {
        return;
    }
    menu.close();
}
