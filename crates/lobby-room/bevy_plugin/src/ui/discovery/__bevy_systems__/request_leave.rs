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

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::input::InputPlugin;
    use bevy::input::keyboard::KeyCode;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::request_leave;

    fn app_closed() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
        app.insert_resource(State::new(ui::discovery::LeaveDialog::Closed));
        app.init_resource::<NextState<ui::discovery::LeaveDialog>>();
        app.add_systems(Update, request_leave);
        app
    }

    fn opening(app: &App) -> bool {
        matches!(
            app.world()
                .resource::<NextState<ui::discovery::LeaveDialog>>(),
            NextState::Pending(ui::discovery::LeaveDialog::Open)
        )
    }

    #[test]
    fn test_usage() {
        let mut clicked = app_closed();
        clicked
            .world_mut()
            .spawn((ui::discovery::LeaveDoor, Pressed));
        clicked.update();
        assert!(opening(&clicked));
    }

    #[test]
    fn a_door_press_opens_the_prompt_once() {
        let mut app = app_closed();
        app.world_mut().spawn((ui::discovery::LeaveDoor, Pressed));
        app.update();
        assert!(opening(&app));
    }

    #[test]
    fn an_escape_press_alone_never_opens_the_prompt() {
        let mut app = app_closed();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(!opening(&app));
    }

    #[test]
    fn a_door_press_keeps_an_open_prompt_open() {
        let mut app = app_closed();
        app.insert_resource(State::new(ui::discovery::LeaveDialog::Open));
        app.world_mut().spawn((ui::discovery::LeaveDoor, Pressed));
        app.update();
        assert!(!opening(&app));
    }
}
