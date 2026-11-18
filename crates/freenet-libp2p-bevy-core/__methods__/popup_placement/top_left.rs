use bevy::math::Vec2;

use crate::ui;

#[must_use]
pub fn top_left(placement: &ui::floating::PopupPlacement) -> Vec2 {
    let side = placement.resolve_side();
    let wanted = match side {
        ui::floating::PopupSide::Below => Vec2::new(
            placement.horizontal_align_start(),
            placement.anchor.max.y + *placement.gap,
        ),
        ui::floating::PopupSide::Above => Vec2::new(
            placement.horizontal_align_start(),
            placement.anchor.min.y - *placement.gap - placement.popup.y,
        ),
        ui::floating::PopupSide::Right => Vec2::new(
            placement.anchor.max.x + *placement.gap,
            placement.vertical_align_start(),
        ),
        ui::floating::PopupSide::Left => Vec2::new(
            placement.anchor.min.x - *placement.gap - placement.popup.x,
            placement.vertical_align_start(),
        ),
    };
    ui::floating::PopupPlacement::clamped_into(placement.area, placement.popup, wanted)
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};

    use super::top_left;
    use crate::ui;

    fn area() -> Rect {
        Rect::from_corners(Vec2::ZERO, Vec2::new(960.0, 540.0))
    }

    fn placement(
        preferred_side: ui::floating::PopupSide,
        cross_align: ui::floating::PopupAlign,
        anchor: Rect,
        popup: Vec2,
    ) -> ui::floating::PopupPlacement {
        ui::floating::PopupPlacement {
            anchor,
            popup,
            area: area(),
            preferred_side,
            cross_align,
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
            ui::floating::PopupAlign::Centre,
            anchor(Vec2::new(100.0, 100.0), Vec2::new(150.0, 120.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(top_left(&placement), Vec2::new(85.0, 124.0));
    }

    #[test]
    fn a_popup_that_fits_below_is_placed_below() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Centre,
            anchor(Vec2::new(400.0, 300.0), Vec2::new(500.0, 320.0)),
            Vec2::new(100.0, 40.0),
        );
        assert_eq!(
            top_left(&placement),
            Vec2::new(
                placement.anchor.center().x - placement.popup.x / 2.0,
                placement.anchor.max.y + *placement.gap,
            )
        );
    }

    #[test]
    fn a_popup_with_no_room_below_flips_above() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Centre,
            anchor(Vec2::new(100.0, 480.0), Vec2::new(150.0, 520.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(top_left(&placement), Vec2::new(85.0, 456.0));
    }

    #[test]
    fn a_popup_with_no_room_left_flips_right() {
        let placement = placement(
            ui::floating::PopupSide::Left,
            ui::floating::PopupAlign::Centre,
            anchor(Vec2::new(10.0, 100.0), Vec2::new(60.0, 120.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(top_left(&placement), Vec2::new(64.0, 100.0));
    }

    #[test]
    fn the_side_with_more_room_wins_when_neither_fits() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Centre,
            anchor(Vec2::new(100.0, 10.0), Vec2::new(150.0, 520.0)),
            Vec2::new(80.0, 20.0),
        );
        assert_eq!(top_left(&placement), Vec2::new(85.0, 520.0));
    }

    #[test]
    fn a_popup_inside_the_area_is_not_moved() {
        let placement = placement(
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Start,
            anchor(Vec2::new(200.0, 200.0), Vec2::new(260.0, 220.0)),
            Vec2::new(60.0, 20.0),
        );
        assert_eq!(top_left(&placement), Vec2::new(200.0, 224.0));
    }
}
