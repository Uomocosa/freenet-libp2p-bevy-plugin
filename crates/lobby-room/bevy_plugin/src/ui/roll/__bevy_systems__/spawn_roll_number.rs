use bevy::prelude::*;

use crate::ui;

use ui::roll::DIGIT_ROWS;

pub fn spawn_roll_number(
    parent: &mut ChildSpawnerCommands,
    pill: Entity,
    slot: usize,
    value: u32,
    style: ui::roll::RollNumberStyle,
) -> Entity {
    let mut window = parent.spawn((
        ui::roll::RollNumber { pill, slot, value },
        Node {
            width: Val::Px(f32::from(style.digit_count) * style.digit_width),
            height: Val::Px(style.row_height),
            overflow: Overflow::clip_y(),
            ..default()
        },
    ));
    let window_id = window.id();
    window.with_children(|columns| {
        for place in 0..style.digit_count {
            let offset = f32::from(ui::roll::State::digit_at(value, usize::from(place)));
            columns
                .spawn((
                    ui::roll::RollDigit {
                        place: usize::from(place),
                        row_height: style.row_height,
                    },
                    UiTransform::from_translation(Val2::px(0.0, -(offset * style.row_height))),
                    Node {
                        width: Val::Px(style.digit_width),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|column| {
                    for row in 0..DIGIT_ROWS {
                        column.spawn((
                            Node {
                                width: Val::Px(style.digit_width),
                                height: Val::Px(style.row_height),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            children![(
                                Text::new((row % 10).to_string()),
                                TextFont {
                                    font_size: FontSize::Px(style.font_size),
                                    ..default()
                                },
                                TextColor(style.text_color),
                            )],
                        ));
                    }
                });
        }
    });
    window_id
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_roll_number;
    use crate::ui;
    use ui::roll::RollNumberStyle;

    const ONE_DIGIT: RollNumberStyle = RollNumberStyle {
        font_size: 11.0,
        digit_width: 6.0,
        row_height: 10.0,
        digit_count: 1,
        text_color: Color::WHITE,
    };

    const TWO_DIGITS: RollNumberStyle = RollNumberStyle {
        digit_count: 2,
        ..ONE_DIGIT
    };

    fn app_with(style: RollNumberStyle, value: u32) -> App {
        let mut app = App::new();
        app.add_systems(Startup, move |mut commands: Commands| {
            let pill = commands.spawn_empty().id();
            commands.entity(pill).with_children(|parent| {
                spawn_roll_number(&mut parent.reborrow(), pill, 0, value, style);
            });
        });
        app.update();
        app
    }

    fn digit_offsets(app: &mut App) -> Vec<f32> {
        let mut query = app
            .world_mut()
            .query::<(&ui::roll::RollDigit, &UiTransform)>();
        let mut offsets: Vec<(usize, f32)> = query
            .iter(app.world())
            .map(|(digit, transform)| {
                let Val::Px(y) = transform.translation.y else {
                    panic!("the digit column translates in pixels")
                };
                (digit.place, y)
            })
            .collect();
        offsets.sort_by_key(|(place, _)| *place);
        offsets.into_iter().map(|(_, y)| y).collect()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(TWO_DIGITS, 9);
        let column_count = {
            let mut numbers = app
                .world_mut()
                .query::<(&ui::roll::RollNumber, &Children)>();
            let (number, columns) = numbers.single(app.world()).expect("one number");
            assert_eq!(number.slot, 0);
            assert_eq!(number.value, 9);
            columns.len()
        };
        assert_eq!(column_count, 2, "one column per digit");
        assert_eq!(digit_offsets(&mut app), vec![-90.0, 0.0]);
    }

    #[test]
    fn every_column_stacks_the_digits_plus_a_trailing_zero() {
        let mut app = app_with(ONE_DIGIT, 0);
        let mut columns = app.world_mut().query::<(&Children, &ui::roll::RollDigit)>();
        for (rows, digit) in columns.iter(app.world()) {
            assert_eq!(rows.len(), ui::roll::DIGIT_ROWS);
            assert_eq!(digit.place, 0);
        }
        let mut rows = app.world_mut().query::<&Text>();
        let glyphs: Vec<String> = rows
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect();
        assert_eq!(glyphs.len(), ui::roll::DIGIT_ROWS);
        assert_eq!(glyphs.last().map(String::as_str), Some("0"));
    }

    #[test]
    fn a_two_digit_number_shows_its_tens_row() {
        let mut app = app_with(TWO_DIGITS, 10);
        assert_eq!(digit_offsets(&mut app), vec![0.0, -10.0]);
    }

    fn preview_widgets(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let pill = commands
            .spawn((
                ChildOf(root),
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    padding: UiRect::axes(Val::Px(7.0), Val::Px(2.5)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.16, 0.2, 0.26)),
            ))
            .id();
        commands.entity(pill).with_children(|parent| {
            spawn_roll_number(&mut parent.reborrow(), pill, 0, 9, ONE_DIGIT);
            spawn_roll_number(&mut parent.reborrow(), pill, 1, 11, TWO_DIGITS);
        });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, preview_widgets);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
    }

    #[test]
    fn spawn_roll_number_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_roll_number"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("a one-digit nine and a two-digit eleven"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
