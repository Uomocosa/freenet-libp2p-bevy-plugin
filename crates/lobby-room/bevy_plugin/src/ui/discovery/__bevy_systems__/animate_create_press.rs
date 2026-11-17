use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::CreateShake;
use ui::discovery::{CREATE_PRESS_SCALE, CREATE_SHAKE_PX, CREATE_SHAKE_SECS, CREATE_SHAKE_TURNS};

type CreateParts<'a> = (
    Entity,
    Option<&'a mut CreateShake>,
    Has<Pressed>,
    &'a mut UiTransform,
);

pub fn animate_create_press(
    time: Res<Time>,
    mut buttons: Query<CreateParts, With<ui::discovery::CreateRoomButton>>,
    mut commands: Commands,
) {
    let delta = time.into_inner().delta_secs();
    for (entity, shake, pressed, mut transform) in &mut buttons {
        let scale = if pressed { CREATE_PRESS_SCALE } else { 1.0 };
        transform.scale = Vec2::splat(scale);
        transform.translation.x = Val::ZERO;
        let Some(mut shake) = shake else {
            continue;
        };
        let elapsed = (**shake + delta).min(CREATE_SHAKE_SECS);
        **shake = elapsed;
        let progress = elapsed / CREATE_SHAKE_SECS;
        let offset = CREATE_SHAKE_PX
            * (1.0 - progress)
            * (progress * CREATE_SHAKE_TURNS * std::f32::consts::TAU).sin();
        transform.translation.x = Val::Px(offset);
        if elapsed >= CREATE_SHAKE_SECS {
            commands.entity(entity).remove::<CreateShake>();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::Pressed;

    use super::animate_create_press;

    fn app_with(pressed: bool, shake: Option<f32>) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        let mut button = app
            .world_mut()
            .spawn((ui::discovery::CreateRoomButton, UiTransform::default()));
        if pressed {
            button.insert(Pressed);
        }
        if let Some(elapsed) = shake {
            button.insert(ui::discovery::CreateShake(elapsed));
        }
        let entity = button.id();
        app.add_systems(Update, animate_create_press);
        (app, entity)
    }

    #[test]
    fn test_usage() {
        let (mut pressed, entity) = app_with(true, None);
        pressed.update();
        let transform = pressed
            .world()
            .get::<UiTransform>(entity)
            .expect("transform");
        assert!((transform.scale.x - ui::discovery::CREATE_PRESS_SCALE).abs() < f32::EPSILON);
        assert_eq!(transform.translation.x, Val::ZERO);

        let (mut shaking, entity) = app_with(false, Some(0.0));
        shaking.update();
        shaking.update();
        let transform = shaking
            .world()
            .get::<UiTransform>(entity)
            .expect("transform");
        assert!((transform.scale.x - 1.0).abs() < f32::EPSILON);
        assert_ne!(transform.translation.x, Val::ZERO);
        for _ in 0..8 {
            shaking.update();
        }
        assert!(
            shaking
                .world()
                .get::<ui::discovery::CreateShake>(entity)
                .is_none(),
            "the shake finishes and removes its state"
        );
        let transform = shaking
            .world()
            .get::<UiTransform>(entity)
            .expect("transform");
        assert_eq!(transform.translation.x, Val::ZERO);
    }
}
