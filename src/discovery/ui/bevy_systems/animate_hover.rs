use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{
    HOVER_OUTLINE_COLOR, HOVER_OUTLINE_WIDTH, HOVER_PER_SEC, HOVER_SCALE,
};
use discovery::ui::{HoverGlow, HoverOutlineOnly};

type Unstyled = (
    With<HoverGlow>,
    Or<(Without<UiTransform>, Without<Outline>)>,
);

type HoverParts<'a> = (
    Option<&'a Interaction>,
    Has<HoverOutlineOnly>,
    &'a mut HoverGlow,
    &'a mut UiTransform,
    &'a mut Outline,
);

pub fn animate_hover(
    time: Res<Time>,
    mut commands: Commands,
    mut fresh: Query<Entity, Unstyled>,
    mut nodes: Query<HoverParts>,
) {
    for entity in &mut fresh {
        commands.entity(entity).insert_if_new((
            UiTransform::default(),
            Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
        ));
    }
    let step = HOVER_PER_SEC * time.into_inner().delta_secs();
    for (interaction, outline_only, mut glow, mut transform, mut outline) in &mut nodes {
        let hovered = interaction.is_some_and(|interaction| *interaction != Interaction::None);
        let target = if hovered { 1.0 } else { 0.0 };
        let progress = if **glow < target {
            (**glow + step).min(target)
        } else {
            (**glow - step).max(target)
        };
        if (**glow - progress).abs() >= f32::EPSILON {
            **glow = progress;
        }
        let grow = if outline_only { 0.0 } else { progress };
        let scale = Vec2::splat((HOVER_SCALE - 1.0).mul_add(grow, 1.0));
        if transform.scale != scale {
            transform.scale = scale;
        }
        let color = HOVER_OUTLINE_COLOR.with_alpha(progress);
        if outline.color != color {
            outline.color = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::animate_hover;
    use crate::discovery;

    fn app_with(interaction: Interaction) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        let entity = app
            .world_mut()
            .spawn((interaction, discovery::ui::HoverGlow::default()))
            .id();
        app.add_systems(Update, animate_hover);
        (app, entity)
    }

    #[test]
    fn test_usage() {
        let (mut hovered, entity) = app_with(Interaction::Hovered);
        for _ in 0..10 {
            hovered.update();
        }
        let glow = **hovered
            .world()
            .get::<discovery::ui::HoverGlow>(entity)
            .expect("glow");
        assert!((glow - 1.0).abs() < f32::EPSILON);
        let scale = hovered
            .world()
            .get::<UiTransform>(entity)
            .expect("transform")
            .scale;
        assert!(scale.x > 1.0);

        let (mut idle, entity) = app_with(Interaction::None);
        for _ in 0..10 {
            idle.update();
        }
        let glow = **idle
            .world()
            .get::<discovery::ui::HoverGlow>(entity)
            .expect("glow");
        assert!(glow.abs() < f32::EPSILON);
    }

    fn spawn_swatch(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            discovery::ui::HoverGlow::default(),
            Interaction::Hovered,
            Node {
                width: Val::Px(180.0),
                height: Val::Px(56.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            children![Text::new("hover me")],
        ));
    }

    fn hover_all(mut nodes: Query<&mut Interaction, With<discovery::ui::HoverGlow>>) {
        for mut interaction in &mut nodes {
            *interaction = Interaction::Hovered;
        }
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_swatch);
        app.add_systems(Update, (hover_all, animate_hover).chain());
    }

    const fn preview_noop(_world: &mut World) {}

    fn scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_hover"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("idle"),
                apply: preview_noop,
            }],
            timeline,
        }
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    #[test]
    fn animate_hover_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("hover_in"),
            frames: 20,
            fps: 30,
            apply: preview_noop,
        };
        lele_bevy_preview::run(&scene(Some(timeline)), &config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }
}
