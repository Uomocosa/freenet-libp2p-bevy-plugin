use crate::ui;
use bevy::prelude::*;

pub fn build_plugin(_plugin: ui::roster::UiPlugin, app: &mut App) {
    use ui::roster::bevy_systems as systems;
    app.init_resource::<ui::roster::MemberIcons>();
    app.init_resource::<ui::roster::ListState>();
    app.add_systems(Startup, systems::spawn_roster_bubble);
    app.add_systems(
        Update,
        (
            systems::prune_member_icons,
            systems::refresh_roster_status,
            systems::refresh_roster_tooltips_text,
            systems::refresh_roster_tooltip,
            systems::toggle_roster_list,
            systems::refresh_roster_list,
        )
            .chain(),
    );
}

#[cfg(test)]
mod tests {
    use super::build_plugin;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        build_plugin(ui::roster::UiPlugin, &mut app);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
    }
}
