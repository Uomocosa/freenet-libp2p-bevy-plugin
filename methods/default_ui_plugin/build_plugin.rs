use bevy::prelude::*;

use crate::discovery;

pub fn build_plugin(_plugin: &discovery::ui::DefaultUiPlugin, app: &mut App) {
    use discovery::ui::bevy_systems as ui;
    use discovery::ui::{LeaveDialog, UiScreen};

    app.add_systems(OnEnter(UiScreen::Connecting), ui::spawn_connecting);
    app.add_systems(OnEnter(UiScreen::Lobby), ui::spawn_root);
    app.add_systems(OnEnter(UiScreen::InRoom), ui::spawn_leave_door);
    app.add_systems(OnEnter(LeaveDialog::Open), ui::spawn_leave_prompt);
    app.add_systems(
        Update,
        (
            ui::pick_screen,
            ui::animate_hover,
            ui::spin_connecting.run_if(in_state(UiScreen::Connecting)),
            (
                (ui::sync_room_list, ui::highlight_current_room).chain(),
                ui::gate_create,
                ui::handle_create,
                ui::handle_join,
            )
                .run_if(in_state(UiScreen::Lobby)),
            (
                ui::request_leave,
                ui::handle_leave_answer.run_if(in_state(LeaveDialog::Open)),
                ui::animate_door,
            )
                .chain()
                .run_if(in_state(UiScreen::InRoom)),
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
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<discovery::Command>();
        app.init_resource::<discovery::Snapshot>();
        build_plugin(&discovery::ui::DefaultUiPlugin, &mut app);
        super::super::finish_plugin(&discovery::ui::DefaultUiPlugin, &mut app);
        app.update();
        let mut spinners = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::SpinnerRing>>();
        assert_eq!(spinners.iter(app.world()).count(), 1);
    }
}
