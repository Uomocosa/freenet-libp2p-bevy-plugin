use bevy::prelude::*;

use crate::discovery;

pub fn build_plugin(_plugin: &discovery::ui::DefaultUiPlugin, app: &mut App) {
    app.add_systems(Startup, discovery::ui::bevy_systems::spawn_root);
    app.add_systems(
        Update,
        (
            discovery::ui::bevy_systems::sync_room_list,
            discovery::ui::bevy_systems::handle_create,
            discovery::ui::bevy_systems::handle_join,
        ),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_plugin;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Command>();
        app.init_resource::<discovery::Snapshot>();
        build_plugin(&discovery::ui::DefaultUiPlugin, &mut app);
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::CreateRoomButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }
}
