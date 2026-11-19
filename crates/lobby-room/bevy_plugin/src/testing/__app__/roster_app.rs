use bevy::prelude::*;

use crate::discovery;
use crate::ui;

pub fn roster_app(roster: discovery::RoomRoster) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(roster);
    app.add_plugins(ui::roster::UiPlugin);
    app
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::discovery;
    use crate::testing;
    use crate::ui;

    use super::roster_app;

    #[test]
    fn test_usage() {
        let mut app = roster_app(testing::room_roster(
            discovery::OwnRoomStatus::Connected,
            1,
            8,
            Vec::new(),
        ));
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
    }
}
