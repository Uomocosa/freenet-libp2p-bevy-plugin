use bevy::prelude::*;

use crate::roster;

pub fn build_plugin(_plugin: roster::UiPlugin, app: &mut App) {
    use roster::bevy_systems as ui;
    app.init_resource::<roster::MemberIcons>();
    app.init_resource::<roster::ListState>();
    app.add_systems(Startup, ui::spawn_roster_bubble);
    app.add_systems(
        Update,
        (
            ui::prune_member_icons,
            ui::refresh_roster_status,
            ui::refresh_roster_tooltips_text,
            ui::refresh_roster_tooltip,
            ui::toggle_roster_list,
            ui::refresh_roster_list,
        )
            .chain(),
    );
}

#[cfg(test)]
mod tests {
    use super::build_plugin;
    use crate::roster;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        build_plugin(roster::UiPlugin, &mut app);
        app.update();
        let mut bubbles = app
            .world_mut()
            .query_filtered::<(), With<roster::RosterBubble>>();
        assert_eq!(bubbles.iter(app.world()).count(), 1);
    }
}
