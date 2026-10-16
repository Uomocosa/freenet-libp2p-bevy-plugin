use bevy::prelude::*;

use crate::discovery;

pub fn build_plugin(_plugin: &discovery::ui::DefaultUiPlugin, app: &mut App) {
    app.init_resource::<discovery::ui::LeavePromptOpen>();
    app.add_systems(
        Startup,
        (
            discovery::ui::bevy_systems::spawn_root,
            discovery::ui::bevy_systems::spawn_leave_door,
        ),
    );
    app.add_systems(
        Update,
        (
            (
                discovery::ui::bevy_systems::sync_room_list,
                discovery::ui::bevy_systems::highlight_current_room,
            )
                .chain(),
            discovery::ui::bevy_systems::gate_create,
            discovery::ui::bevy_systems::animate_hover,
            discovery::ui::bevy_systems::handle_create,
            discovery::ui::bevy_systems::handle_join,
            discovery::ui::bevy_systems::sync_in_room,
            (
                discovery::ui::bevy_systems::request_leave,
                discovery::ui::bevy_systems::handle_leave_answer,
                discovery::ui::bevy_systems::sync_leave_prompt,
                discovery::ui::bevy_systems::animate_door,
            )
                .chain()
                .after(discovery::ui::bevy_systems::sync_in_room),
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
