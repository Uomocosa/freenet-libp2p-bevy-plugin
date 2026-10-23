use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{DOOR_OPEN_PER_SEC, DOOR_SWING};
use discovery::ui::{DoorLabel, DoorOpening, DoorPanel, LeaveDialog, LeaveDoor};

pub fn animate_door(
    time: Res<Time>,
    dialog: Option<Res<State<LeaveDialog>>>,
    door: Single<(&Interaction, &mut DoorOpening), With<LeaveDoor>>,
    mut panel: Single<&mut Node, With<DoorPanel>>,
    mut label: Single<&mut TextColor, With<DoorLabel>>,
) {
    let (interaction, mut opening) = door.into_inner();
    let dialog_open = dialog.is_some_and(|dialog| *dialog.get() == LeaveDialog::Open);
    let wants_open = dialog_open || *interaction != Interaction::None;
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
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::animate_door;
    use crate::discovery;

    fn hovered_door() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(100),
        ));
        app.world_mut().spawn((
            discovery::ui::LeaveDoor,
            discovery::ui::DoorOpening::default(),
            Interaction::Hovered,
        ));
        app.world_mut()
            .spawn((discovery::ui::DoorPanel, Node::default()));
        app.world_mut()
            .spawn((discovery::ui::DoorLabel, TextColor(Color::WHITE)));
        app.add_systems(Update, animate_door);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = hovered_door();
        for _ in 0..10 {
            app.update();
        }
        let mut doors = app.world_mut().query::<&discovery::ui::DoorOpening>();
        let opening = doors.single(app.world()).map(|opening| **opening).ok();
        assert_eq!(opening, Some(1.0));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                discovery::ui::bevy_systems::spawn_leave_door,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
        app.add_systems(Update, animate_door);
    }

    const fn preview_noop(_world: &mut World) {}

    fn open_prompt(world: &mut World) {
        world.insert_resource(State::new(discovery::ui::LeaveDialog::Open));
    }

    fn opened(world: &mut World) {
        open_prompt(world);
        let mut doors = world.query::<&mut discovery::ui::DoorOpening>();
        for mut opening in doors.iter_mut(world) {
            **opening = 1.0;
        }
    }

    fn scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_door"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: opened,
                },
            ],
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
    fn animate_door_ui_png_preview() {
        scene(None)
            .run(&config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn animate_door_ui_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("swing_open"),
            frames: 20,
            fps: 30,
            apply: open_prompt,
        };
        scene(Some(timeline))
            .run(&config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }
}
