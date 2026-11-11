use bevy::prelude::*;

use crate::direct_dial;
use direct_dial::{
    DRAWER_BACKDROP_ALPHA, DRAWER_SLIDE_SECS, DRAWER_WIDTH, FriendsDrawer, FriendsDrawerBackdrop,
    FriendsDrawerPanel, FriendsDrawerSlide, FriendsDrawerState,
};

type SlideParts<'a> = (&'a mut FriendsDrawerSlide, &'a mut UiTransform);

pub fn animate_friends_drawer(
    time: Res<Time>,
    state: Res<FriendsDrawerState>,
    root: Single<&mut Visibility, With<FriendsDrawer>>,
    panel: Single<SlideParts, With<FriendsDrawerPanel>>,
    backdrop: Single<&mut BackgroundColor, With<FriendsDrawerBackdrop>>,
) {
    let state = state.into_inner();
    let target = if **state { DRAWER_SLIDE_SECS } else { 0.0 };
    let mut root = root.into_inner();
    let (mut slide, mut transform) = panel.into_inner();
    let mut backdrop = backdrop.into_inner();
    let delta = time.into_inner().delta_secs();
    if **slide < target {
        **slide = (**slide + delta).min(target);
    } else if **slide > target {
        **slide = (**slide - delta).max(target);
    }
    let progress = (**slide / DRAWER_SLIDE_SECS).clamp(0.0, 1.0);
    let wanted = Val2::px((1.0 - progress) * -DRAWER_WIDTH, 0.0);
    if transform.translation != wanted {
        transform.translation = wanted;
    }
    let wanted_backdrop = Color::srgba(0.0, 0.0, 0.0, DRAWER_BACKDROP_ALPHA * progress);
    let BackgroundColor(current) = &mut *backdrop;
    if *current != wanted_backdrop {
        *current = wanted_backdrop;
    }
    *root = if progress > 0.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::animate_friends_drawer;
    use crate::basic::constants::{DRAWER_PANEL_COLOR, ROOM_COLOR};
    use crate::direct_dial;
    use crate::discovery;
    use direct_dial::{
        DRAWER_BACKDROP_ALPHA, DRAWER_SLIDE_SECS, DRAWER_WIDTH, FriendsDrawer,
        FriendsDrawerBackdrop, FriendsDrawerPanel, FriendsDrawerSlide, FriendsDrawerState,
    };

    fn app_with(open: bool, slide: f32) -> (App, Entity, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ));
        app.insert_resource(FriendsDrawerState(open));
        let root = app
            .world_mut()
            .spawn((FriendsDrawer, Visibility::Hidden))
            .id();
        let panel = app
            .world_mut()
            .spawn((
                ChildOf(root),
                FriendsDrawerPanel,
                FriendsDrawerSlide(slide),
                UiTransform::default(),
            ))
            .id();
        let backdrop = app
            .world_mut()
            .spawn((
                ChildOf(root),
                FriendsDrawerBackdrop,
                BackgroundColor::default(),
            ))
            .id();
        app.add_systems(Update, animate_friends_drawer);
        (app, root, panel, backdrop)
    }

    #[test]
    fn test_usage() {
        let (mut app, root, _panel, _backdrop) = app_with(false, 0.0);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Hidden
        );
    }

    #[test]
    fn an_open_drawer_slides_towards_full_progress() {
        let (mut app, root, panel, backdrop) = app_with(true, 0.0);
        for _ in 0..2 {
            app.update();
        }
        let slide = *app.world().get::<FriendsDrawerSlide>(panel).expect("slide");
        assert!(*slide > 0.0, "the slide advanced");
        let transform = app.world().get::<UiTransform>(panel).expect("transform");
        let Val::Px(x) = transform.translation.x else {
            panic!("the panel translates in pixels");
        };
        assert!(
            x > -DRAWER_WIDTH && x < 0.0,
            "the panel is partially slid in, x={x}"
        );
        let BackgroundColor(color) = *app.world().get::<BackgroundColor>(backdrop).expect("bg");
        assert!(color.alpha() > 0.0 && color.alpha() < DRAWER_BACKDROP_ALPHA);
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Visible
        );
    }

    #[test]
    fn a_fully_open_drawer_rests_at_full_progress() {
        let (mut app, root, panel, backdrop) = app_with(true, DRAWER_SLIDE_SECS);
        for _ in 0..2 {
            app.update();
        }
        let transform = app.world().get::<UiTransform>(panel).expect("transform");
        assert_eq!(transform.translation, Val2::ZERO);
        let BackgroundColor(color) = *app.world().get::<BackgroundColor>(backdrop).expect("bg");
        assert!((color.alpha() - DRAWER_BACKDROP_ALPHA).abs() < f32::EPSILON);
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Visible
        );
    }

    #[test]
    fn a_closing_drawer_slides_back_out_and_hides() {
        let (mut app, root, panel, _backdrop) = app_with(false, DRAWER_SLIDE_SECS);
        for _ in 0..2 {
            app.update();
        }
        let slide = *app.world().get::<FriendsDrawerSlide>(panel).expect("slide");
        assert!(*slide < DRAWER_SLIDE_SECS, "the slide retreated");
        let transform = app.world().get::<UiTransform>(panel).expect("transform");
        let Val::Px(x) = transform.translation.x else {
            panic!("the panel translates in pixels");
        };
        assert!(
            x < 0.0 && x > -DRAWER_WIDTH,
            "the panel is partially slid out, x={x}"
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Visible
        );
        for _ in 0..30 {
            app.update();
        }
        let transform = app.world().get::<UiTransform>(panel).expect("transform");
        let Val::Px(x) = transform.translation.x else {
            panic!("the panel translates in pixels");
        };
        assert!(
            (x + DRAWER_WIDTH).abs() < f32::EPSILON,
            "the panel is fully out, x={x}"
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Hidden
        );
    }

    fn spawn_drawer(mut commands: Commands) {
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
                    children![Text::new("the screen behind the drawer")],
                ),
                (
                    FriendsDrawer,
                    GlobalZIndex(direct_dial::DRAWER_Z),
                    Visibility::Hidden,
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    children![
                        (
                            FriendsDrawerBackdrop,
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ),
                        (
                            FriendsDrawerPanel,
                            FriendsDrawerSlide(0.0),
                            UiTransform::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::ZERO,
                                bottom: Val::ZERO,
                                left: Val::ZERO,
                                width: Val::Px(DRAWER_WIDTH),
                                ..default()
                            },
                            BackgroundColor(DRAWER_PANEL_COLOR),
                            children![Text::new("drawer")],
                        ),
                    ],
                ),
            ],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(FriendsDrawerState(false));
        app.add_systems(
            Startup,
            (
                spawn_drawer,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, animate_friends_drawer);
    }

    const fn preview_noop(_world: &mut World) {}

    fn open(world: &mut World) {
        world.insert_resource(FriendsDrawerState(true));
        let mut panels =
            world.query_filtered::<&mut FriendsDrawerSlide, With<FriendsDrawerPanel>>();
        if let Ok(mut slide) = panels.single_mut(world) {
            **slide = DRAWER_SLIDE_SECS;
        }
    }

    #[test]
    fn animate_friends_drawer_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_friends_drawer"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: open,
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }

    #[test]
    fn animate_friends_drawer_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_friends_drawer"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("closed"),
                apply: preview_noop,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("slide in"),
                frames: 20,
                fps: 30,
                apply: open,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
