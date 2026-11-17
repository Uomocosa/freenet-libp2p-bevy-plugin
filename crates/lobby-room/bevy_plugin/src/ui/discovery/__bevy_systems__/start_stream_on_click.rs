use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{RoomButton, StreamedRoom};

pub fn start_stream_on_click(
    buttons: Query<&RoomButton, Added<Pressed>>,
    mut streamed: ResMut<StreamedRoom>,
) {
    for button in &buttons {
        **streamed = Some((**button).clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::start_stream_on_click;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ui::discovery::StreamedRoom>();
        app.add_systems(Update, start_stream_on_click);
        app
    }

    fn press(app: &mut App, room: &str) {
        app.world_mut().spawn((
            ui::discovery::RoomButton(core_discovery::RoomName::from(room)),
            Pressed,
        ));
    }

    fn streamed(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::StreamedRoom>()).clone()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        press(&mut app, "alpha");
        app.update();
        assert_eq!(
            streamed(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn a_second_click_replaces_the_streamed_room() {
        let mut app = app();
        press(&mut app, "alpha");
        app.update();
        press(&mut app, "beta");
        app.update();
        assert_eq!(streamed(&app), Some(core_discovery::RoomName::from("beta")));
    }
}
