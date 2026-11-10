use bevy::prelude::*;

use crate::discovery;

pub fn finish_plugin(_plugin: discovery::DefaultUiPlugin, app: &mut App) {
    app.init_state::<discovery::Screen>();
    app.add_sub_state::<discovery::LeaveDialog>();
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::finish_plugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        finish_plugin(discovery::DefaultUiPlugin, &mut app);
        app.update();
        let screen = *app.world().resource::<State<discovery::Screen>>().get();
        assert_eq!(screen, discovery::Screen::Connecting);
    }
}
