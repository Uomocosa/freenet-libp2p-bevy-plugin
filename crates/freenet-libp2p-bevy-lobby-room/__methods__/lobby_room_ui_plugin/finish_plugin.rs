use crate::ui;
use bevy::prelude::*;

pub fn finish_plugin(_plugin: ui::discovery::LobbyRoomUiPlugin, app: &mut App) {
    app.init_state::<ui::discovery::Screen>();
    app.add_sub_state::<ui::discovery::LeaveDialog>();
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::finish_plugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        finish_plugin(ui::discovery::LobbyRoomUiPlugin, &mut app);
        app.update();
        let screen = *app.world().resource::<State<ui::discovery::Screen>>().get();
        assert_eq!(screen, ui::discovery::Screen::Connecting);
    }
}
