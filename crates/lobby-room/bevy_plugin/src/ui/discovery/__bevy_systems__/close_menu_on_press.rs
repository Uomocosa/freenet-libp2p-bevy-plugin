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

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::close_menu_on_press;
    use ui::discovery::{EscMenuBackdrop, EscMenuResumeButton, EscMenuState};

    fn app_with(open: bool) -> App {
        let mut app = App::new();
        app.insert_resource(EscMenuState(open));
        app.add_systems(Update, close_menu_on_press);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuBackdrop, Pressed));
        app.update();
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn a_backdrop_press_closes_the_menu() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuBackdrop, Pressed));
        app.update();
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn a_resume_press_closes_the_menu() {
        let mut app = app_with(true);
        app.world_mut().spawn((EscMenuResumeButton, Pressed));
        app.update();
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn without_a_press_the_menu_stays_open() {
        let mut app = app_with(true);
        app.world_mut().spawn(EscMenuBackdrop);
        app.update();
        assert!(**app.world().resource::<EscMenuState>());
    }
}
