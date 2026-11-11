use bevy::prelude::*;

use crate::direct_dial;
use direct_dial::{COPY_TOAST_BG_COLOR, COPY_TOAST_FADE_SECS, COPY_TOAST_TEXT_COLOR, CopyToast};

pub fn animate_copy_toast(
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut CopyToast, &mut TextColor, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    let delta = time.into_inner().delta_secs();
    for (entity, mut toast, mut text, mut background) in &mut toasts {
        **toast -= delta;
        if **toast <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (**toast / COPY_TOAST_FADE_SECS).min(1.0);
        let wanted_text = COPY_TOAST_TEXT_COLOR.with_alpha(alpha);
        let TextColor(current) = &mut *text;
        if *current != wanted_text {
            *current = wanted_text;
        }
        let wanted_pill = COPY_TOAST_BG_COLOR.with_alpha(COPY_TOAST_BG_COLOR.alpha() * alpha);
        let BackgroundColor(pill) = &mut *background;
        if *pill != wanted_pill {
            *pill = wanted_pill;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::animate_copy_toast;
    use crate::basic::constants::ROOM_COLOR;
    use crate::direct_dial;
    use crate::discovery;
    use direct_dial::{
        COPY_TOAST_BG_COLOR, COPY_TOAST_LINGER_SECS, COPY_TOAST_TEXT_COLOR, CopyToast,
    };

    fn app_with(remaining: f32) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(200),
        ));
        let toast = app
            .world_mut()
            .spawn((
                CopyToast(remaining),
                Text::new("Copied to clipboard"),
                TextColor(COPY_TOAST_TEXT_COLOR),
                BackgroundColor(COPY_TOAST_BG_COLOR),
            ))
            .id();
        app.add_systems(Update, animate_copy_toast);
        (app, toast)
    }

    #[test]
    fn test_usage() {
        let (mut app, toast) = app_with(COPY_TOAST_LINGER_SECS);
        for _ in 0..2 {
            app.update();
        }
        assert!(app.world().get_entity(toast).is_ok());
    }

    #[test]
    fn the_toast_fades_in_its_last_seconds() {
        let (mut app, toast) = app_with(0.8);
        for _ in 0..3 {
            app.update();
        }
        let TextColor(color) = *app.world().get::<TextColor>(toast).expect("text color");
        assert!(color.alpha() < 1.0, "the text fades");
        let BackgroundColor(background) = *app
            .world()
            .get::<BackgroundColor>(toast)
            .expect("background");
        assert!(
            background.alpha() < COPY_TOAST_BG_COLOR.alpha(),
            "the pill fades"
        );
    }

    #[test]
    fn a_fresh_toast_is_fully_opaque() {
        let (mut app, toast) = app_with(COPY_TOAST_LINGER_SECS);
        for _ in 0..2 {
            app.update();
        }
        let TextColor(color) = *app.world().get::<TextColor>(toast).expect("text color");
        assert!((color.alpha() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn the_toast_despawns_when_its_time_is_up() {
        let (mut app, toast) = app_with(0.5);
        for _ in 0..4 {
            app.update();
        }
        assert!(app.world().get_entity(toast).is_err());
    }

    fn spawn_toast(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ROOM_COLOR),
            children![
                (
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(40.0),
                        right: Val::Px(40.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(discovery::PANEL_COLOR),
                    children![Text::new("the screen behind the toast")],
                ),
                (
                    CopyToast(COPY_TOAST_LINGER_SECS),
                    Text::new("Copied to clipboard"),
                    TextColor(COPY_TOAST_TEXT_COLOR),
                    BackgroundColor(COPY_TOAST_BG_COLOR),
                    UiTransform {
                        translation: Val2 {
                            x: Val::Percent(-50.0),
                            y: Val::ZERO,
                        },
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(28.0),
                        left: Val::Percent(50.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                )
            ],
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_toast,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, animate_copy_toast);
    }

    fn restart_toast(world: &mut World) {
        let mut toasts = world.query_filtered::<&mut CopyToast, ()>();
        if let Ok(mut toast) = toasts.single_mut(world) {
            **toast = COPY_TOAST_LINGER_SECS;
        }
    }

    #[test]
    fn animate_copy_toast_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_copy_toast"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("fresh toast"),
                apply: restart_toast,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("fade out"),
                frames: 52,
                fps: 30,
                apply: restart_toast,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
