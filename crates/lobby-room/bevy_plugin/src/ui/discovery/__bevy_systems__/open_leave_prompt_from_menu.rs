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

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::open_leave_prompt_from_menu;
    use ui::discovery::{EscMenuLeaveButton, EscMenuState, LeaveDialog};

    fn app_with(open: bool) -> App {
        let mut app = App::new();
        app.init_resource::<NextState<LeaveDialog>>();
        app.insert_resource(State::new(LeaveDialog::Closed));
        app.init_resource::<EscMenuState>();
        **app.world_mut().resource_mut::<EscMenuState>() = open;
        app.add_systems(Update, open_leave_prompt_from_menu);
        app
    }

    fn is_opening(app: &App) -> bool {
        matches!(
            app.world().resource::<NextState<LeaveDialog>>(),
            NextState::Pending(LeaveDialog::Open)
        )
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuLeaveButton, Pressed));
        app.update();
        assert!(is_opening(&app));
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn a_leave_press_closes_the_menu_and_opens_the_prompt() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuLeaveButton, Pressed));
        app.update();
        assert!(is_opening(&app));
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn without_a_press_the_prompt_stays_closed() {
        let mut app = app_with(true);
        app.world_mut().spawn(EscMenuLeaveButton);
        app.update();
        assert!(!is_opening(&app));
        assert!(**app.world().resource::<EscMenuState>());
    }
}
