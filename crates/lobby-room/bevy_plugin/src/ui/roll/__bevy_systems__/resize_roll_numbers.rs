use bevy::prelude::*;
use bevy::ui::ComputedNode;

use crate::ui;

pub fn resize_roll_numbers(
    machines: Query<&ui::roll::RollMachine>,
    mut numbers: Query<(&ui::roll::RollNumber, &Children, &mut Node)>,
    columns: Query<&Children, With<ui::roll::RollNumberColumn>>,
    rows: Query<(&ui::roll::RollNumberRow, &ComputedNode)>,
) {
    for (number, window_children, mut node) in &mut numbers {
        let Ok(machine) = machines.get(number.pill) else {
            continue;
        };
        let progress = match &machine.phase {
            ui::roll::Phase::Roll { elapsed_secs, .. } => {
                (elapsed_secs / ui::roll::ROLL_SECS).clamp(0.0, 1.0)
            }
            ui::roll::Phase::Idle | ui::roll::Phase::Reveal { .. } => 1.0,
        };
        let Some(column) = window_children.iter().next() else {
            continue;
        };
        let Ok(column_children) = columns.get(column) else {
            continue;
        };
        let mut outgoing_width = 0.0;
        let mut incoming_width = 0.0;
        for child in column_children.iter() {
            let Ok((row, computed)) = rows.get(child) else {
                continue;
            };
            match row {
                ui::roll::RollNumberRow::Outgoing => outgoing_width = computed.size().x,
                ui::roll::RollNumberRow::Incoming => incoming_width = computed.size().x,
            }
        }
        let width = match (outgoing_width > 0.0, incoming_width > 0.0) {
            (false, false) => None,
            (false, true) => Some(incoming_width),
            (true, false) => Some(outgoing_width),
            (true, true) => {
                Some((incoming_width - outgoing_width).mul_add(progress, outgoing_width))
            }
        };
        let Some(width) = width else {
            continue;
        };
        if progress >= 1.0 && node.width == Val::Px(width) {
            continue;
        }
        node.width = Val::Px(width);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::ComputedNode;

    use super::resize_roll_numbers;
    use crate::ui;
    use ui::roll::NumberStyle;

    const ROW_HEIGHT: f32 = 13.0;
    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: ROW_HEIGHT,
        text_color: Color::WHITE,
    };

    fn app_with_widths(outgoing: f32, incoming: f32, phase: &ui::roll::Phase) -> App {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let pill = commands
                .spawn(ui::roll::RollMachine(ui::roll::State::new(vec![9], None)))
                .id();
            commands.entity(pill).with_children(|parent| {
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut parent.reborrow(),
                    pill,
                    0,
                    9,
                    STYLE,
                );
            });
        });
        app.update();
        {
            let mut machines = app.world_mut().query::<&mut ui::roll::RollMachine>();
            for mut machine in machines.iter_mut(app.world_mut()) {
                machine.phase = phase.clone();
            }
        }
        let mut rows = app
            .world_mut()
            .query::<(Entity, &ui::roll::RollNumberRow)>();
        let found: Vec<(Entity, ui::roll::RollNumberRow)> = rows
            .iter(app.world())
            .map(|(entity, row)| (entity, *row))
            .collect();
        for (entity, row) in found {
            let width = match row {
                ui::roll::RollNumberRow::Outgoing => outgoing,
                ui::roll::RollNumberRow::Incoming => incoming,
            };
            app.world_mut().entity_mut(entity).insert(ComputedNode {
                size: Vec2::new(width, ROW_HEIGHT),
                ..default()
            });
        }
        app.add_systems(Update, resize_roll_numbers);
        app.update();
        app
    }

    fn window_width(app: &mut App) -> f32 {
        let mut numbers = app.world_mut().query::<(&ui::roll::RollNumber, &Node)>();
        let (_, node) = numbers.single(app.world()).expect("one number");
        let Val::Px(width) = node.width else {
            panic!("the window width is in pixels")
        };
        width
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_widths(6.0, 12.0, &ui::roll::Phase::Idle);
        assert_eq!(window_width(&mut app), 12.0);
    }

    #[test]
    fn half_way_sits_between_the_two_text_widths() {
        let phase = ui::roll::Phase::Roll {
            from: vec![9],
            to: vec![10],
            elapsed_secs: ui::roll::ROLL_SECS / 2.0,
        };
        let mut app = app_with_widths(6.0, 12.0, &phase);
        assert_eq!(window_width(&mut app), 9.0);
    }

    #[test]
    fn a_settled_number_keeps_the_outgoing_width_when_the_incoming_is_unmeasured() {
        let mut app = app_with_widths(6.0, 0.0, &ui::roll::Phase::Idle);
        assert_eq!(window_width(&mut app), 6.0);
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct WidthWritten(bool);

    fn record_width_write(
        numbers: Query<Ref<Node>, With<ui::roll::RollNumber>>,
        mut written: ResMut<WidthWritten>,
    ) {
        **written = numbers.iter().any(|node| node.is_changed());
    }

    #[test]
    fn a_settled_number_keeps_its_matching_width_unwritten() {
        let mut app = app_with_widths(6.0, 12.0, &ui::roll::Phase::Idle);
        assert_eq!(window_width(&mut app), 12.0);
        app.init_resource::<WidthWritten>();
        app.add_systems(PostUpdate, record_width_write);
        app.update();
        app.update();
        assert!(
            !**app.world().resource::<WidthWritten>(),
            "a settled number whose width already matches is left alone"
        );
    }

    fn spawn_resize_preview(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let pill = commands
            .spawn((
                ChildOf(root),
                ui::roll::RollMachine(ui::roll::State::new(vec![9], None)),
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.16, 0.44, 0.24)),
            ))
            .id();
        commands.entity(pill).with_children(|parent| {
            ui::roll::bevy_systems::spawn_roll_number(&mut parent.reborrow(), pill, 0, 9, STYLE);
        });
    }

    fn preview_show_ten(
        time: Res<Time>,
        mut machines: Query<&mut ui::roll::RollMachine>,
        mut started: Local<bool>,
    ) {
        if time.into_inner().elapsed_secs() < 0.3 || *started {
            return;
        }
        *started = true;
        for mut machine in &mut machines {
            let _ = machine.update(ui::roll::Input::Show { numbers: vec![10] });
        }
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_resize_preview);
        app.add_systems(
            Update,
            (
                preview_show_ten,
                ui::roll::bevy_systems::drive_roll_machines,
            )
                .chain(),
        );
        app.add_systems(
            PostUpdate,
            resize_roll_numbers.after(bevy::ui::UiSystems::Layout),
        );
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn resize_roll_numbers_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("resize_roll_numbers"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("after the roll"),
                apply: preview_noop,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("nine to ten window width"),
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
