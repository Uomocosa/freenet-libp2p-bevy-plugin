use crate::ui;
use bevy::prelude::*;

use ui::direct_dial::{
    DRAWER_BACKDROP_ALPHA, DRAWER_WIDTH, FriendsDrawer, FriendsDrawerBackdrop, FriendsDrawerPanel,
    FriendsDrawerSlide, FriendsDrawerState,
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
    let target = if **state {
        ui::constants::DRAWER_SLIDE_SECS
    } else {
        0.0
    };
    let mut root = root.into_inner();
    let (mut slide, mut transform) = panel.into_inner();
    let mut backdrop = backdrop.into_inner();
    let delta = time.into_inner().delta_secs();
    if **slide < target {
        **slide = (**slide + delta).min(target);
    } else if **slide > target {
        **slide = (**slide - delta).max(target);
    }
    let progress = (**slide / ui::constants::DRAWER_SLIDE_SECS).clamp(0.0, 1.0);
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
    use crate::ui;
    use bevy::prelude::*;
    use lele_bevy_preview::preview;
    use lele_bevy_preview::scene::{self, Kind, Scene, State};

    use super::animate_friends_drawer;
    use ui::direct_dial::{
        DRAWER_WIDTH, DRAWER_Z, FriendsDrawer, FriendsDrawerBackdrop, FriendsDrawerPanel,
        FriendsDrawerSlide, FriendsDrawerState,
    };

    #[test]
    fn animate_friends_drawer_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        Scene {
            name: String::from("animate_friends_drawer"),
            kind: Kind::System,
            build: |app: &mut App| {
                app.insert_resource(FriendsDrawerState(false));
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands.spawn(Scene::centered_root()).id();
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
                                children![Text::new("the screen behind the drawer")],
                            ),
                            (
                                FriendsDrawer,
                                GlobalZIndex(DRAWER_Z),
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
                                            width: Val::Px(DRAWER_WIDTH),
                                            ..default()
                                        },
                                        BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
                                        children![Text::new("drawer")],
                                    ),
                                ],
                            ),
                        ],
                    ));
                });
                app.add_systems(PostStartup, scene::bevy_systems::center_roots);
                app.add_systems(Update, animate_friends_drawer);
            },
            states: vec![
                State {
                    label: String::from("closed"),
                    apply: |_world: &mut World| {},
                },
                State {
                    label: String::from("open"),
                    apply: |world: &mut World| {
                        world.insert_resource(FriendsDrawerState(true));
                        let mut panels = world
                            .query_filtered::<&mut FriendsDrawerSlide, With<FriendsDrawerPanel>>();
                        if let Ok(mut slide) = panels.single_mut(world) {
                            **slide = ui::constants::DRAWER_SLIDE_SECS;
                        }
                    },
                },
            ],
            timeline: None,
        }
        .run(&preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }

    #[test]
    fn animate_friends_drawer_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        Scene {
            name: String::from("animate_friends_drawer"),
            kind: Kind::System,
            build: |app: &mut App| {
                app.insert_resource(FriendsDrawerState(false));
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands.spawn(Scene::centered_root()).id();
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
                                children![Text::new("the screen behind the drawer")],
                            ),
                            (
                                FriendsDrawer,
                                GlobalZIndex(DRAWER_Z),
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
                                            width: Val::Px(DRAWER_WIDTH),
                                            ..default()
                                        },
                                        BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
                                        children![Text::new("drawer")],
                                    ),
                                ],
                            ),
                        ],
                    ));
                });
                app.add_systems(PostStartup, scene::bevy_systems::center_roots);
                app.add_systems(Update, animate_friends_drawer);
            },
            states: vec![State {
                label: String::from("closed"),
                apply: |_world: &mut World| {},
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("slide in"),
                frames: 20,
                fps: 30,
                apply: |world: &mut World| {
                    world.insert_resource(FriendsDrawerState(true));
                    let mut panels =
                        world.query_filtered::<&mut FriendsDrawerSlide, With<FriendsDrawerPanel>>();
                    if let Ok(mut slide) = panels.single_mut(world) {
                        **slide = ui::constants::DRAWER_SLIDE_SECS;
                    }
                },
            }),
        }
        .run(&preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
