use crate::ui;

#[must_use]
pub fn resolve_side(placement: &ui::floating::PopupPlacement) -> ui::floating::PopupSide {
    let preferred = placement.preferred_side;
    let opposite = preferred.opposite();
    let preferred_room = placement.room_on(preferred);
    let opposite_room = placement.room_on(opposite);
    let needed = placement.extent_on(preferred) + *placement.gap;
    if preferred_room >= needed || preferred_room >= opposite_room {
        preferred
    } else {
        opposite
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};

    use super::resolve_side;
    use crate::ui;

    fn placement(
        preferred_side: ui::floating::PopupSide,
        anchor: Rect,
        popup: Vec2,
    ) -> ui::floating::PopupPlacement {
        ui::floating::PopupPlacement {
            anchor,
            popup,
            area: Rect::from_corners(Vec2::ZERO, Vec2::new(960.0, 540.0)),
            preferred_side,
            cross_align: ui::floating::PopupAlign::Centre,
            gap: ui::floating::PopupGap(4.0),
        }
    }

    fn anchor(min: Vec2, max: Vec2) -> Rect {
        Rect::from_corners(min, max)
    }

    #[test]
    fn test_usage() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            anchor(Vec2::new(100.0, 100.0), Vec2::new(150.0, 120.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(resolve_side(&placement), ui::floating::PopupSide::Below);
    }

    #[test]
    fn a_popup_with_no_room_below_flips_above() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            anchor(Vec2::new(100.0, 480.0), Vec2::new(150.0, 520.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(resolve_side(&placement), ui::floating::PopupSide::Above);
    }

    #[test]
    fn a_popup_with_no_room_left_flips_right() {
        let placement = placement(
            ui::floating::PopupSide::Left,
            anchor(Vec2::new(10.0, 100.0), Vec2::new(60.0, 120.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(resolve_side(&placement), ui::floating::PopupSide::Right);
    }

    #[test]
    fn the_side_with_more_room_wins_when_neither_fits() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            anchor(Vec2::new(100.0, 10.0), Vec2::new(150.0, 520.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(resolve_side(&placement), ui::floating::PopupSide::Below);
    }
}
