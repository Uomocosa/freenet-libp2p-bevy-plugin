use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{LeaveDialog, LeaveNoButton, LeaveYesButton};

pub fn handle_leave_answer(
    yes: Query<&Interaction, (Changed<Interaction>, With<LeaveYesButton>)>,
    no: Query<&Interaction, (Changed<Interaction>, With<LeaveNoButton>)>,
    mut next: ResMut<NextState<LeaveDialog>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    let pressed = |interaction: &Interaction| *interaction == Interaction::Pressed;
    let confirmed = yes.iter().any(pressed);
    let cancelled = no.iter().any(pressed);
    if confirmed {
        tracing::info!(target: "discovery", "ui leave confirmed");
        commands.write(discovery::Command::Leave);
    }
    if confirmed || cancelled {
        next.set(LeaveDialog::Closed);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::handle_leave_answer;
    use crate::discovery;

    fn sent_after(marker: impl Component) -> (Vec<discovery::Command>, bool) {
        let mut app = App::new();
        app.add_message::<discovery::Command>();
        app.init_resource::<NextState<discovery::ui::LeaveDialog>>();
        app.world_mut().spawn((marker, Interaction::Pressed));
        app.add_systems(Update, handle_leave_answer);
        app.update();
        let messages = app.world().resource::<Messages<discovery::Command>>();
        let mut cursor = messages.get_cursor();
        let sent = cursor.read(messages).cloned().collect();
        let closing = matches!(
            app.world()
                .resource::<NextState<discovery::ui::LeaveDialog>>(),
            NextState::Pending(discovery::ui::LeaveDialog::Closed)
        );
        (sent, closing)
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            sent_after(discovery::ui::LeaveYesButton),
            (vec![discovery::Command::Leave], true)
        );
        assert_eq!(sent_after(discovery::ui::LeaveNoButton), (Vec::new(), true));
    }
}
