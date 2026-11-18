use crate::ui;
use bevy::prelude::*;

use ui::direct_dial::{
    COPY_TOAST_BG_COLOR, COPY_TOAST_FADE_SECS, COPY_TOAST_TEXT_COLOR, CopyToast,
};

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
    use crate::ui;
    use bevy::prelude::*;

    use super::animate_copy_toast;
    use ui::direct_dial::{
        COPY_TOAST_BG_COLOR, COPY_TOAST_LINGER_SECS, COPY_TOAST_TEXT_COLOR, CopyToast,
    };

    #[test]
    fn animate_copy_toast_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_copy_toast"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
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
                                BackgroundColor(ui::constants::ROOM_COLOR),
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
                                        BackgroundColor(ui::discovery::PANEL_COLOR),
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
                                    ),
                                ],
                            ));
                        },
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(Update, animate_copy_toast);
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("fresh toast"),
                apply: |world: &mut World| {
                    let mut toasts = world.query_filtered::<&mut CopyToast, ()>();
                    if let Ok(mut toast) = toasts.single_mut(world) {
                        **toast = COPY_TOAST_LINGER_SECS;
                    }
                },
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("fade out"),
                frames: 52,
                fps: 30,
                apply: |world: &mut World| {
                    let mut toasts = world.query_filtered::<&mut CopyToast, ()>();
                    if let Ok(mut toast) = toasts.single_mut(world) {
                        **toast = COPY_TOAST_LINGER_SECS;
                    }
                },
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
