use bevy::prelude::{App, MinimalPlugins};
use bevy::state::app::StatesPlugin;

use crate::discovery;
use crate::ui;

pub fn lobby_ui_app(
    state: Option<discovery::state_machine::DiscoveryState>,
    rows: Option<discovery::LobbyRows>,
) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.add_message::<discovery::Command>();
    if let Some(state) = state {
        app.insert_resource(state);
    }
    if let Some(rows) = rows {
        app.insert_resource(rows);
    }
    app.add_plugins(ui::discovery::LobbyRoomUiPlugin);
    app
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Plugin, State};

    use crate::ui;

    use super::lobby_ui_app;

    #[test]
    fn test_usage() {
        let mut app = lobby_ui_app(None, None);
        Plugin::finish(&ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        assert!(
            app.world()
                .get_resource::<State<ui::discovery::Screen>>()
                .is_some()
        );
    }
}
