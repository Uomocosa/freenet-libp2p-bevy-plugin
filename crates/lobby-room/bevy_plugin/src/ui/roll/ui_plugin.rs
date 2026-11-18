use bevy::prelude::*;

use crate::ui;

pub struct UiPlugin;

impl bevy::prelude::Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ui::roll::bevy_systems::drive_roll_machines.in_set(ui::roll::RollSet::Drive),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::UiPlugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(UiPlugin);
        app.update();
        assert!(app.world().resource::<Time>().delta_secs() >= 0.0);
    }
}
