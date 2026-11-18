use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{LeaveDialog, LeaveNoButton, LeaveYesButton};

pub fn handle_leave_answer(
    yes: Query<(), (Added<Pressed>, With<LeaveYesButton>)>,
    no: Query<(), (Added<Pressed>, With<LeaveNoButton>)>,
    mut next: ResMut<NextState<LeaveDialog>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    let confirmed = yes.iter().next().is_some();
    let cancelled = no.iter().next().is_some();
    if confirmed {
        tracing::info!(target: "discovery", "ui leave confirmed");
        commands.write(core_discovery::Command::Leave);
    }
    if confirmed || cancelled {
        next.set(LeaveDialog::Closed);
    }
}
