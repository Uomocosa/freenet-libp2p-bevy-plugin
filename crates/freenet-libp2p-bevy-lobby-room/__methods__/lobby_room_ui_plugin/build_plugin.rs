use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use freenet_libp2p_bevy_core::room_sync;

pub fn build_plugin(_plugin: ui::discovery::LobbyRoomUiPlugin, app: &mut App) {
    use ui::discovery::bevy_systems as systems;
    use ui::discovery::{EscMenuState, JoinError, LeaveDialog, Screen};

    app.add_plugins(ui::roster::UiPlugin);
    app.init_resource::<JoinError>();
    app.init_resource::<EscMenuState>();
    app.init_resource::<ui::discovery::FriendsPressBeat>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_message::<core_discovery::Event>();
    app.add_message::<room_sync::JoinFailure>();
    app.add_systems(OnEnter(Screen::Connecting), systems::spawn_connecting);
    app.add_systems(OnEnter(Screen::Lobby), systems::spawn_root);
    app.add_systems(OnEnter(Screen::JoiningRoom), systems::spawn_joining_room);
    app.add_systems(OnEnter(Screen::JoinFailed), systems::spawn_join_failed);
    app.add_systems(OnEnter(Screen::InRoom), systems::spawn_leave_door);
    app.add_systems(OnEnter(LeaveDialog::Open), systems::spawn_leave_prompt);
    app.add_systems(Startup, systems::spawn_esc_menu);
    app.add_systems(
        Update,
        (
            (
                systems::record_join_error,
                systems::record_join_failure,
                systems::pick_screen,
            )
                .chain(),
            (
                systems::handle_menu_keys,
                systems::close_menu_on_press,
                systems::start_friends_beat,
                systems::open_leave_prompt_from_menu,
                systems::finish_friends_beat,
            )
                .chain(),
            systems::animate_hover,
            systems::animate_esc_menu,
            systems::refresh_menu_leave_button,
            systems::spin_connecting.run_if(in_state(Screen::Connecting)),
            systems::dismiss_join_failed.run_if(in_state(Screen::JoinFailed)),
            (
                (
                    systems::rebuild_room_list,
                    systems::highlight_current_room,
                    systems::scroll_room_names,
                )
                    .chain(),
                systems::gate_create,
                systems::handle_create,
                systems::handle_join,
            )
                .run_if(in_state(Screen::Lobby)),
            (
                systems::request_leave,
                systems::handle_leave_answer.run_if(in_state(LeaveDialog::Open)),
                systems::animate_door,
            )
                .chain()
                .run_if(in_state(Screen::InRoom)),
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::build_plugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_message::<core_discovery::Command>();
        build_plugin(ui::discovery::LobbyRoomUiPlugin, &mut app);
        super::super::finish_plugin(ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        let mut spinners = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::SpinnerRing>>();
        assert_eq!(spinners.iter(app.world()).count(), 1);
    }
}
