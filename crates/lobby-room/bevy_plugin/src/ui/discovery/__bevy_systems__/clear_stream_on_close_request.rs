use crate::ui;
use bevy::prelude::*;

use ui::discovery::{CloseStream, StreamedRoom};

pub fn clear_stream_on_close_request(
    mut requests: MessageReader<CloseStream>,
    mut streamed: ResMut<StreamedRoom>,
) {
    if requests.read().next().is_some() {
        **streamed = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::clear_stream_on_close_request;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ui::discovery::CloseStream>();
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.add_systems(Update, clear_stream_on_close_request);
        app
    }

    fn streamed(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::StreamedRoom>()).clone()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        app.world_mut().write_message(ui::discovery::CloseStream);
        app.update();
        assert_eq!(streamed(&app), None);
    }
}
