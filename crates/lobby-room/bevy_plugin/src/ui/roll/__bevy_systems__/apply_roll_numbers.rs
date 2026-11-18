use bevy::prelude::*;

use crate::ui;

pub fn apply_roll_numbers(
    pill: Entity,
    from: &[u32],
    to: &[u32],
    progress: f32,
    numbers: &mut Query<(&mut ui::roll::RollNumber, &Children)>,
    digits: &mut Query<(&ui::roll::RollDigit, &mut UiTransform)>,
) {
    for (mut number, children) in numbers.iter_mut() {
        if number.pill != pill {
            continue;
        }
        let (Some(&start), Some(&end)) = (from.get(number.slot), to.get(number.slot)) else {
            continue;
        };
        for column in children.iter() {
            let Ok((digit, mut transform)) = digits.get_mut(column) else {
                continue;
            };
            let position = ui::roll::State::roll_digit_position(start, end, digit.place, progress);
            transform.translation = Val2::px(0.0, -(position * digit.row_height));
        }
        if progress >= 1.0 {
            number.value = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::apply_roll_numbers;
    use crate::ui;
    use ui::roll::RollNumberStyle;

    #[derive(Component)]
    struct Pill;

    const STYLE: RollNumberStyle = RollNumberStyle {
        font_size: 11.0,
        digit_width: 6.0,
        row_height: 10.0,
        digit_count: 2,
        text_color: Color::WHITE,
    };

    fn apply_half_way(
        mut numbers: Query<(&mut ui::roll::RollNumber, &Children)>,
        mut digits: Query<(&ui::roll::RollDigit, &mut UiTransform)>,
        pills: Query<Entity, With<Pill>>,
    ) {
        for pill in &pills {
            apply_roll_numbers(pill, &[9], &[10], 0.5, &mut numbers, &mut digits);
        }
    }

    fn app_with_nine() -> App {
        let mut app = App::new();
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
        app.add_systems(Update, apply_half_way);
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
        let mut app = app_with_nine();
        let offsets = digit_offsets(&mut app);
        assert_eq!(offsets.len(), 2);
        assert!(
            offsets[1] < 0.0,
            "the tens column slid halfway from the zero row"
        );
    }

    #[test]
    fn halfway_from_nine_to_ten_slides_the_ones_column_past_the_trailing_zero() {
        let mut app = app_with_nine();
        let offsets = digit_offsets(&mut app);
        assert!((offsets[0] + 95.0).abs() < 0.01, "ones: {}", offsets[0]);
        assert!((offsets[1] + 5.0).abs() < 0.01, "tens: {}", offsets[1]);
    }
}
