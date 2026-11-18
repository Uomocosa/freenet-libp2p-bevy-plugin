use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{LeaveDialog, LeaveDoor};

pub fn request_leave(
    doors: Query<(), (Added<Pressed>, With<LeaveDoor>)>,
    dialog: Res<State<LeaveDialog>>,
    mut next: ResMut<NextState<LeaveDialog>>,
) {
    if doors.is_empty() {
        return;
    }
    let is_open = *dialog.into_inner().get() == LeaveDialog::Open;
    if !is_open {
        tracing::info!(target: "discovery", "ui leave prompt open=true");
        next.set(LeaveDialog::Open);
    }
}
