use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::discovery;
use discovery::ui::SpinnerRing;
use discovery::ui::basic::constants::SPIN_TURNS_PER_SEC;

pub fn spin_connecting(time: Res<Time>, mut rings: Query<&mut UiTransform, With<SpinnerRing>>) {
    let angle = time.into_inner().elapsed_secs() * SPIN_TURNS_PER_SEC * TAU;
    for mut transform in &mut rings {
        transform.rotation = Rot2::radians(angle);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::spin_connecting;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ));
        let ring = app
            .world_mut()
            .spawn((discovery::ui::SpinnerRing, UiTransform::default()))
            .id();
        app.add_systems(Update, spin_connecting);
        app.update();
        app.update();
        let rotation = app.world().get::<UiTransform>(ring).expect("ring").rotation;
        assert_ne!(rotation, Rot2::IDENTITY);
    }
}
