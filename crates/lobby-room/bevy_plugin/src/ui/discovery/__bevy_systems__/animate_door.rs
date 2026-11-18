use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{DOOR_OPEN_PER_SEC, DOOR_SWING};
use ui::discovery::{DoorLabel, DoorOpening, DoorPanel, LeaveDialog, LeaveDoor};

type DoorParts<'a> = (Option<&'a Hovered>, Has<Pressed>, &'a mut DoorOpening);

pub fn animate_door(
    time: Res<Time>,
    dialog: Option<Res<State<LeaveDialog>>>,
    door: Single<DoorParts<'_>, With<LeaveDoor>>,
    mut panel: Single<&mut Node, With<DoorPanel>>,
    mut label: Single<&mut TextColor, With<DoorLabel>>,
) {
    let (hovered, pressed, mut opening) = door.into_inner();
    let dialog_open = dialog.is_some_and(|dialog| *dialog.get() == LeaveDialog::Open);
    let wants_open = dialog_open || hovered.is_some_and(Hovered::get) || pressed;
    let target = if wants_open { 1.0 } else { 0.0 };
    let step = DOOR_OPEN_PER_SEC * time.into_inner().delta_secs();
    let progress = if **opening < target {
        (**opening + step).min(target)
    } else {
        (**opening - step).max(target)
    };
    if (**opening - progress).abs() >= f32::EPSILON {
        **opening = progress;
    }
    let width = Val::Percent(100.0 * DOOR_SWING.mul_add(-progress, 1.0));
    if panel.width != width {
        panel.width = width;
    }
    let TextColor(color) = &mut **label;
    let faded = color.with_alpha(progress);
    if *color != faded {
        *color = faded;
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn animate_door_ui_png_preview() {
        let scene = lele_bevy_preview::scene::Scene {
            name: String::from("animate_door"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(
                    Startup,
                    (
                        ui::discovery::bevy_systems::spawn_leave_door,
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(Update, super::animate_door);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: |world: &mut World| {
                        world.insert_resource(State::new(ui::discovery::LeaveDialog::Open));
                        let mut doors = world.query::<&mut ui::discovery::DoorOpening>();
                        for mut opening in doors.iter_mut(world) {
                            **opening = 1.0;
                        }
                    },
                },
            ],
            timeline: None,
        };
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        scene
            .run(&lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            })
            .expect("preview");
    }

    #[test]
    fn animate_door_ui_mp4_preview() {
        let scene = lele_bevy_preview::scene::Scene {
            name: String::from("animate_door"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(
                    Startup,
                    (
                        ui::discovery::bevy_systems::spawn_leave_door,
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
                app.add_systems(Update, super::animate_door);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: |world: &mut World| {
                        world.insert_resource(State::new(ui::discovery::LeaveDialog::Open));
                        let mut doors = world.query::<&mut ui::discovery::DoorOpening>();
                        for mut opening in doors.iter_mut(world) {
                            **opening = 1.0;
                        }
                    },
                },
            ],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("swing_open"),
                frames: 20,
                fps: 30,
                apply: |world: &mut World| {
                    world.insert_resource(State::new(ui::discovery::LeaveDialog::Open));
                },
            }),
        };
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        scene
            .run(&lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            })
            .expect("preview");
    }
}
