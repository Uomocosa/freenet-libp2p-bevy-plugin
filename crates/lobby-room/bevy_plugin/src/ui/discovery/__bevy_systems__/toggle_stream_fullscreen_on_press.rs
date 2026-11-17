use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{RoomStreamFullscreenButton, StreamDisplay};

pub fn toggle_stream_fullscreen_on_press(
    presses: Query<(), (Added<Pressed>, With<RoomStreamFullscreenButton>)>,
    mut display: ResMut<StreamDisplay>,
) {
    if presses.is_empty() {
        return;
    }
    display.toggle();
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::toggle_stream_fullscreen_on_press;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<ui::discovery::StreamDisplay>();
        app.add_systems(Update, toggle_stream_fullscreen_on_press);
        app
    }

    fn press(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((ui::discovery::RoomStreamFullscreenButton, Pressed))
            .id()
    }

    fn display(app: &App) -> ui::discovery::StreamDisplay {
        *app.world().resource::<ui::discovery::StreamDisplay>()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        press(&mut app);
        app.update();
        assert_eq!(display(&app), ui::discovery::StreamDisplay::Fullscreen);
    }

    #[test]
    fn a_second_press_returns_to_the_panel() {
        let mut app = app();
        let button = press(&mut app);
        app.update();
        app.world_mut().entity_mut(button).remove::<Pressed>();
        app.update();
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        assert_eq!(display(&app), ui::discovery::StreamDisplay::Panel);
    }

    #[test]
    fn a_press_on_another_button_changes_nothing() {
        let mut app = app();
        app.world_mut()
            .spawn((ui::discovery::RoomStreamCloseButton, Pressed));
        app.update();
        assert_eq!(display(&app), ui::discovery::StreamDisplay::Panel);
    }
}
