use bevy::prelude::*;

use crate::discovery;
use discovery::ui::{LeaveDialog, LeaveDoor};

pub fn request_leave(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    doors: Query<&Interaction, (Changed<Interaction>, With<LeaveDoor>)>,
    dialog: Res<State<LeaveDialog>>,
    mut next: ResMut<NextState<LeaveDialog>>,
) {
    let is_open = *dialog.into_inner().get() == LeaveDialog::Open;
    let clicked = doors
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let escaped = keys.is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
    let open = match (clicked, escaped) {
        (true, _) => true,
        (false, true) => !is_open,
        (false, false) => return,
    };
    if is_open != open {
        tracing::info!(target: "discovery", "ui leave prompt open={open}");
        next.set(if open {
            LeaveDialog::Open
        } else {
            LeaveDialog::Closed
        });
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::request_leave;
    use crate::discovery;

    fn app_closed() -> App {
        let mut app = App::new();
        app.insert_resource(State::new(discovery::ui::LeaveDialog::Closed));
        app.init_resource::<NextState<discovery::ui::LeaveDialog>>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, request_leave);
        app
    }

    fn opening(app: &App) -> bool {
        matches!(
            app.world()
                .resource::<NextState<discovery::ui::LeaveDialog>>(),
            NextState::Pending(discovery::ui::LeaveDialog::Open)
        )
    }

    #[test]
    fn test_usage() {
        let mut clicked = app_closed();
        clicked
            .world_mut()
            .spawn((discovery::ui::LeaveDoor, Interaction::Pressed));
        clicked.update();
        assert!(opening(&clicked));

        let mut escaped = app_closed();
        escaped
            .world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        escaped.update();
        assert!(opening(&escaped));
    }
}
