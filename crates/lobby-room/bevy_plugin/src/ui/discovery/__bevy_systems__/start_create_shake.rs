use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::CreateShake;

pub fn start_create_shake(
    pressed: Query<Entity, (Added<Pressed>, With<ui::discovery::CreateRoomButton>)>,
    mut commands: Commands,
) {
    for entity in &pressed {
        commands.entity(entity).insert(CreateShake::default());
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::start_create_shake;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let button = app
            .world_mut()
            .spawn((ui::discovery::CreateRoomButton, Pressed))
            .id();
        let idle = app
            .world_mut()
            .spawn((ui::discovery::CreateRoomButton,))
            .id();
        app.add_systems(Update, start_create_shake);
        app.update();
        assert!(
            app.world()
                .get::<ui::discovery::CreateShake>(button)
                .is_some(),
            "the pressed create button starts its shake"
        );
        assert!(
            app.world()
                .get::<ui::discovery::CreateShake>(idle)
                .is_none(),
            "an idle create button does not shake"
        );
    }
}
