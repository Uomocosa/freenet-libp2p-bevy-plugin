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

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_leave_answer;

    fn sent_after(marker: impl Component) -> (Vec<core_discovery::Command>, bool) {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        app.init_resource::<NextState<ui::discovery::LeaveDialog>>();
        app.world_mut().spawn((marker, Pressed));
        app.add_systems(Update, handle_leave_answer);
        app.update();
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        let sent = cursor.read(messages).cloned().collect();
        let closing = matches!(
            app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Closed)
        );
        (sent, closing)
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            sent_after(ui::discovery::LeaveYesButton),
            (vec![core_discovery::Command::Leave], true)
        );
        assert_eq!(sent_after(ui::discovery::LeaveNoButton), (Vec::new(), true));
    }
}
