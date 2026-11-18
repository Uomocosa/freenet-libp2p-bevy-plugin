use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{EscMenuLeaveButton, EscMenuState, LeaveDialog};

pub fn open_leave_prompt_from_menu(
    presses: Query<(), (Added<Pressed>, With<EscMenuLeaveButton>)>,
    mut menu: ResMut<EscMenuState>,
    mut next: ResMut<NextState<LeaveDialog>>,
) {
    if presses.is_empty() {
        return;
    }
    menu.close();
    tracing::info!(target: "discovery", "ui leave prompt open=true");
    next.set(LeaveDialog::Open);
}
