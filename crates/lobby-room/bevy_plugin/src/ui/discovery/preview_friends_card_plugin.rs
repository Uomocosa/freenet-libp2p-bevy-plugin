use crate::ui;
use bevy::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct PreviewFriendsCardPlugin;

impl bevy::prelude::Plugin for PreviewFriendsCardPlugin {
    fn build(&self, app: &mut App) {
        use ui::discovery::Screen;
        use ui::discovery::bevy_systems as systems;

        app.add_systems(
            OnEnter(Screen::Lobby),
            systems::spawn_preview_friends_card.after(systems::spawn_room_preview),
        );
        app.add_systems(
            Update,
            systems::rebuild_preview_friends
                .after(systems::show_room_bubble_tooltip)
                .run_if(in_state(Screen::Lobby)),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<ui::discovery::Screen>();
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.world_mut().spawn(ui::discovery::RoomPreviewViewport);
        app.add_plugins(ui::discovery::PreviewFriendsCardPlugin);
        app.world_mut()
            .resource_mut::<NextState<ui::discovery::Screen>>()
            .set(ui::discovery::Screen::Lobby);
        app.update();
        app.update();
        let mut cards = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomPreviewFriends>>();
        assert_eq!(cards.iter(app.world()).count(), 1);
    }
}
