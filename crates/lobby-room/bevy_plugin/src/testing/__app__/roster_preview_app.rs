use bevy::prelude::*;

use crate::discovery;
use crate::testing;
use crate::ui;

pub fn roster_preview_app(app: &mut App) {
    app.insert_resource(testing::room_roster(
        discovery::OwnRoomStatus::Connected,
        3,
        8,
        vec![
            testing::room_roster_entry(
                "12D3KooWcarol0Zm9c",
                "carol",
                discovery::MemberRoomStatus::Connecting,
                false,
            ),
            testing::room_roster_entry(
                "12D3KooWbob000Kd2a",
                "bob",
                discovery::MemberRoomStatus::Connected,
                false,
            ),
            testing::room_roster_entry("me", "alice", discovery::MemberRoomStatus::Connected, true),
        ],
    ));
    app.add_plugins(ui::roster::UiPlugin);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::ui;

    use super::roster_preview_app;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        roster_preview_app(&mut app);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
    }
}
