use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::prelude::{App, Assets, Image, Update};

use crate::discovery;
use crate::testing;
use crate::ui;

pub fn lobby_preview_app(
    app: &mut App,
    state: discovery::state_machine::DiscoveryState,
    rows: discovery::LobbyRows,
    is_friends_card_shown: bool,
) {
    app.init_resource::<Assets<Image>>();
    app.add_message::<discovery::Command>();
    app.insert_resource(state);
    app.insert_resource(rows);
    app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
    if is_friends_card_shown {
        app.add_plugins(ui::discovery::PreviewFriendsCardPlugin);
    }
    app.add_systems(
        Update,
        testing::bevy_systems::pin_hovered
            .before(ui::discovery::bevy_systems::track_room_preview)
            .before(ui::discovery::bevy_systems::animate_hover)
            .before(ui::discovery::bevy_systems::show_room_bubble_tooltip)
            .before(ui::discovery::bevy_systems::show_room_search_tooltip),
    );
    app.add_systems(
        Update,
        (
            ui::discovery::bevy_systems::show_preview_image::<testing::FakeReplica<u8, u8>>,
            ui::discovery::bevy_systems::show_stream_image::<testing::FakeReplica<u8, u8>>,
        ),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Plugin;
    use bevy::state::app::StatesPlugin;

    use crate::testing;
    use crate::ui;

    use super::lobby_preview_app;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new().with_plugin(StatesPlugin);
        lobby_preview_app(
            &mut app,
            testing::preview_snapshot(),
            testing::preview_rows(),
            true,
        );
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.tick(5);
        assert!(testing::find_entity::<ui::discovery::RoomPreviewPanel>(app.world_mut()).is_some());
    }
}
