use bevy::prelude::*;

use crate::discovery;

pub fn finish_plugin(_plugin: &discovery::ui::DefaultUiPlugin, app: &mut App) {
    app.init_state::<discovery::ui::Screen>();
    app.add_sub_state::<discovery::ui::LeaveDialog>();
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::finish_plugin;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        finish_plugin(&discovery::ui::DefaultUiPlugin, &mut app);
        app.update();
        let screen = *app.world().resource::<State<discovery::ui::Screen>>().get();
        assert_eq!(screen, discovery::ui::Screen::Connecting);
    }
}
