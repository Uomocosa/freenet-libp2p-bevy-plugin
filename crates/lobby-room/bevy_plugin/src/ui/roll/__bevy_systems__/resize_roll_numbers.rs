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
    use super::resize_roll_numbers;
    use crate::ui;
    use bevy::prelude::*;
    use ui::roll::NumberStyle;

    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: 13.0,
        text_color: Color::WHITE,
    };

    #[test]
    fn resize_roll_numbers_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("resize_roll_numbers"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
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
                        ui::roll::bevy_systems::spawn_roll_number(
                            &mut parent.reborrow(),
                            pill,
                            0,
                            9,
                            STYLE,
                        );
                    });
                });
                app.add_systems(
                    Update,
                    (
                        |time: Res<Time>,
                         mut machines: Query<&mut ui::roll::RollMachine>,
                         mut started: Local<bool>| {
                            if time.into_inner().elapsed_secs() < 0.3 || *started {
                                return;
                            }
                            *started = true;
                            for mut machine in &mut machines {
                                let _ = machine.update(ui::roll::Input::Show { numbers: vec![10] });
                            }
                        },
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
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("after the roll"),
                apply: |_world: &mut World| {},
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("nine to ten window width"),
                frames: 60,
                fps: 30,
                apply: |_world: &mut World| {},
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
