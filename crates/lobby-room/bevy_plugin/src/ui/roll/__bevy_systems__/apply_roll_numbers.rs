use bevy::prelude::*;

use crate::ui;

pub fn apply_roll_numbers(
    pill: Entity,
    from: &[u32],
    to: &[u32],
    progress: f32,
    numbers: &mut Query<(&mut ui::roll::RollNumber, &Children)>,
    columns: &mut Query<(&Children, &mut UiTransform), With<ui::roll::RollNumberColumn>>,
    rows: &mut Query<(&ui::roll::RollNumberRow, &mut Text)>,
) {
    let eased = ui::roll::bevy_systems::ease_out_cubic(progress);
    let is_settled = progress >= 1.0;
    for (mut number, window_children) in numbers.iter_mut() {
        if number.pill != pill {
            continue;
        }
        let (Some(&start), Some(&end)) = (from.get(number.slot), to.get(number.slot)) else {
            continue;
        };
        let Some(column) = window_children.iter().next() else {
            continue;
        };
        let Ok((column_children, mut transform)) = columns.get_mut(column) else {
            continue;
        };
        for child in column_children.iter() {
            let Ok((row, mut text)) = rows.get_mut(child) else {
                continue;
            };
            let wanted = match *row {
                ui::roll::RollNumberRow::Outgoing if is_settled => end,
                ui::roll::RollNumberRow::Outgoing => start,
                ui::roll::RollNumberRow::Incoming => end,
            }
            .to_string();
            if text.as_str() != wanted {
                *text = Text::new(wanted);
            }
        }
        let offset = if is_settled {
            0.0
        } else {
            -(eased * number.row_height)
        };
        transform.translation = Val2::px(0.0, offset);
        if is_settled {
            number.value = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use derive_more::Deref;

    use super::apply_roll_numbers;
    use crate::ui;
    use ui::roll::NumberStyle;

    #[derive(Component)]
    struct Pill;

    #[derive(Resource, Deref)]
    struct Progress(f32);

    const ROW_HEIGHT: f32 = 10.0;
    const STYLE: NumberStyle = NumberStyle {
        font_size: 11.0,
        row_height: ROW_HEIGHT,
        text_color: Color::WHITE,
    };

    fn apply_at_progress(
        progress: Res<Progress>,
        mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
        mut columns: Query<(&Children, &mut UiTransform), With<ui::roll::RollNumberColumn>>,
        mut rows: Query<(&ui::roll::RollNumberRow, &mut Text)>,
        pills: Query<Entity, With<Pill>>,
    ) {
        let progress = **progress.into_inner();
        for pill in &pills {
            apply_roll_numbers(
                pill,
                &[9],
                &[10],
                progress,
                &mut numbers,
                &mut columns,
                &mut rows,
            );
        }
    }

    fn app_with_nine(progress: f32) -> App {
        let mut app = App::new();
        app.insert_resource(Progress(progress));
        app.add_systems(Startup, |mut commands: Commands| {
            let pill = commands.spawn(Pill).id();
            commands.entity(pill).with_children(|pill_children| {
                ui::roll::bevy_systems::spawn_roll_number(
                    &mut pill_children.reborrow(),
                    pill,
                    0,
                    9,
                    STYLE,
                );
            });
        });
        app.add_systems(Update, apply_at_progress);
        app.update();
        app
    }

    fn column_offset(app: &mut App) -> f32 {
        let mut query = app
            .world_mut()
            .query::<(&UiTransform, &ui::roll::RollNumberColumn)>();
        let (transform, _) = query.single(app.world()).expect("one column");
        let Val::Px(y) = transform.translation.y else {
            panic!("the column translates in pixels")
        };
        y
    }

    fn row_texts(app: &mut App) -> [String; 2] {
        let mut query = app.world_mut().query::<(&ui::roll::RollNumberRow, &Text)>();
        let mut outgoing = String::new();
        let mut incoming = String::new();
        for (row, text) in query.iter(app.world()) {
            match row {
                ui::roll::RollNumberRow::Outgoing => outgoing = text.as_str().to_string(),
                ui::roll::RollNumberRow::Incoming => incoming = text.as_str().to_string(),
            }
        }
        [outgoing, incoming]
    }

    fn number_value(app: &mut App) -> u32 {
        let mut numbers = app.world_mut().query::<&ui::roll::RollNumber>();
        numbers.single(app.world()).expect("one number").value
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_nine(0.5);
        assert_eq!(row_texts(&mut app), [String::from("9"), String::from("10")]);
        assert!((column_offset(&mut app) + 8.75).abs() < 0.01);
        assert_eq!(number_value(&mut app), 9);
    }

    #[test]
    fn settling_copies_the_incoming_number_onto_the_outgoing_row() {
        let mut app = app_with_nine(1.0);
        assert_eq!(column_offset(&mut app), 0.0);
        assert_eq!(
            row_texts(&mut app),
            [String::from("10"), String::from("10")]
        );
        assert_eq!(number_value(&mut app), 10);
    }

    #[derive(Resource, Default, Deref, DerefMut)]
    struct PreviewProgress(f32);

    fn preview_apply(
        progress: Res<PreviewProgress>,
        mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
        mut columns: Query<(&Children, &mut UiTransform), With<ui::roll::RollNumberColumn>>,
        mut rows: Query<(&ui::roll::RollNumberRow, &mut Text)>,
        pills: Query<Entity, With<Pill>>,
    ) {
        let progress = **progress.into_inner();
        for pill in &pills {
            apply_roll_numbers(
                pill,
                &[9],
                &[10],
                progress,
                &mut numbers,
                &mut columns,
                &mut rows,
            );
        }
    }

    fn spawn_preview_pill(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let pill = commands
            .spawn((
                ChildOf(root),
                Pill,
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

    fn preview_start(world: &mut World) {
        world.insert_resource(PreviewProgress(0.0));
    }

    fn preview_half(world: &mut World) {
        world.insert_resource(PreviewProgress(0.5));
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<PreviewProgress>();
        app.add_systems(Startup, spawn_preview_pill);
        app.add_systems(Update, preview_apply);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn apply_roll_numbers_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("apply_roll_numbers"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("start"),
                    apply: preview_start,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("halfway"),
                    apply: preview_half,
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
}
