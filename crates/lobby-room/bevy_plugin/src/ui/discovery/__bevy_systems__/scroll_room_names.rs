use crate::ui;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

use ui::discovery::{ROOM_NAME_SCROLL_PER_SEC, RoomNameClip, RoomNameScroller};

pub fn scroll_room_names(
    time: Res<Time>,
    mut scrollers: Query<(&RoomNameScroller, &ChildOf, &Children, &mut UiTransform)>,
    clips: Query<&ComputedNode, With<RoomNameClip>>,
    nodes: Query<&ComputedNode>,
) {
    let elapsed_secs = time.into_inner().elapsed_secs();
    for (_scroller, parent, children, mut transform) in &mut scrollers {
        let Some(clip_width) = clips
            .get(parent.get())
            .ok()
            .filter(|clip| clip.size.x > 0.0)
            .map(|clip| clip.size.x)
        else {
            continue;
        };
        let text_width = children
            .iter()
            .filter_map(|child| nodes.get(child).ok())
            .filter(|text| text.size.x > 0.0)
            .map(|text| text.size.x)
            .next()
            .unwrap_or(0.0);
        let wanted = if text_width > clip_width {
            let travel = text_width + clip_width;
            Val2::px(
                clip_width - (ROOM_NAME_SCROLL_PER_SEC * elapsed_secs).rem_euclid(travel),
                0.0,
            )
        } else {
            Val2::ZERO
        };
        if transform.translation != wanted {
            transform.translation = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use crate::ui::basic;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::scroll_room_names;

    fn app_with(clip_width: f32, text_width: f32) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(500),
        ));
        let clip = app
            .world_mut()
            .spawn((ui::discovery::RoomNameClip, measured(clip_width, 18.0)))
            .id();
        let scroller = app
            .world_mut()
            .spawn((
                ChildOf(clip),
                ui::discovery::RoomNameScroller,
                UiTransform::default(),
            ))
            .id();
        app.world_mut()
            .spawn((ChildOf(scroller), measured(text_width, 16.0)));
        app.add_systems(Update, scroll_room_names);
        (app, clip, scroller)
    }

    fn measured(width: f32, height: f32) -> ComputedNode {
        ComputedNode {
            size: Vec2::new(width, height),
            content_size: Vec2::ZERO,
            scrollbar_size: Vec2::ZERO,
            scroll_position: Vec2::ZERO,
            outline_width: 0.0,
            outline_offset: 0.0,
            unrounded_size: Vec2::new(width, height),
            border: BorderRect::ZERO,
            border_radius: ResolvedBorderRadius::ZERO,
            padding: BorderRect::ZERO,
            inverse_scale_factor: 1.0,
            em_size: EmSize(12.0),
            rem_size: RemSize(12.0),
        }
    }

    #[test]
    fn test_usage() {
        let (mut app, _clip, scroller) = app_with(128.0, 300.0);
        for _ in 0..10 {
            app.update();
        }
        let translation = app
            .world()
            .get::<UiTransform>(scroller)
            .expect("transform")
            .translation;
        let Val::Px(x) = translation.x else {
            panic!("translate is in pixels")
        };
        assert!(x <= 128.0 && x > -300.0);
        assert!(x < 127.0, "the text has moved left");
    }

    #[test]
    fn a_name_that_fits_never_moves() {
        let (mut app, _clip, scroller) = app_with(128.0, 60.0);
        for _ in 0..10 {
            app.update();
        }
        let translation = app
            .world()
            .get::<UiTransform>(scroller)
            .expect("transform")
            .translation;
        assert_eq!(translation, Val2::ZERO);
    }

    #[test]
    fn the_scroll_loops_within_one_text_travel() {
        let (mut app, _clip, scroller) = app_with(128.0, 300.0);
        let mut seen_low = false;
        let mut seen_high = false;
        for _ in 0..400 {
            app.update();
            let Val::Px(x) = app
                .world()
                .get::<UiTransform>(scroller)
                .expect("transform")
                .translation
                .x
            else {
                panic!("translate is in pixels")
            };
            if x < 1.0 {
                seen_low = true;
            }
            if x > 126.0 {
                seen_high = true;
            }
        }
        assert!(seen_low, "the text exits on the left");
        assert!(seen_high, "the text re-enters from the right");
    }

    #[test]
    fn unmeasured_nodes_stay_put() {
        let (mut app, _clip, scroller) = app_with(0.0, 300.0);
        for _ in 0..5 {
            app.update();
        }
        let translation = app
            .world()
            .get::<UiTransform>(scroller)
            .expect("transform")
            .translation;
        assert_eq!(translation, Val2::ZERO);
    }

    fn spawn_scrolling_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(basic::constants::ROOM_COLOR),
            children![(
                ui::discovery::RoomNameClip,
                Node {
                    width: Val::Px(ui::discovery::ROOM_NAME_WIDTH),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                children![(
                    ui::discovery::RoomNameScroller,
                    UiTransform::default(),
                    Node::default(),
                    children![(
                        Text::new("very-long-room-name-that-never-fits".to_string()),
                        TextLayout::no_wrap(),
                    )],
                )],
            )],
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_scrolling_rows);
        app.add_systems(Update, scroll_room_names);
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn scroll_room_names_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("scroll_room_names"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("clipped name"),
                apply: preview_noop,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("scroll"),
                frames: 60,
                fps: 30,
                apply: preview_noop,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
