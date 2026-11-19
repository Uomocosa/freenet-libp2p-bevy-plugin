use bevy::math::{Rect, Vec2};

use crate::ui;

#[must_use]
pub const fn placement(
    area: Rect,
    anchor: Rect,
    popup: Vec2,
    preferred_side: ui::floating::PopupSide,
    cross_align: ui::floating::PopupAlign,
) -> ui::floating::PopupPlacement {
    ui::floating::PopupPlacement {
        anchor,
        popup,
        area,
        preferred_side,
        cross_align,
        gap: ui::floating::PopupGap(4.0),
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};

    use super::placement;
    use crate::ui;

    #[test]
    fn test_usage() {
        let placement = placement(
            Rect::from_corners(Vec2::ZERO, Vec2::new(960.0, 540.0)),
            Rect::from_corners(Vec2::new(90.0, 95.0), Vec2::new(110.0, 105.0)),
            Vec2::new(40.0, 18.0),
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Centre,
        );
        assert_eq!(
            placement.area,
            Rect::from_corners(Vec2::ZERO, Vec2::new(960.0, 540.0))
        );
        assert_eq!(
            placement.anchor,
            Rect::from_corners(Vec2::new(90.0, 95.0), Vec2::new(110.0, 105.0))
        );
        assert_eq!(placement.popup, Vec2::new(40.0, 18.0));
        assert_eq!(placement.preferred_side, ui::floating::PopupSide::Below);
        assert_eq!(placement.cross_align, ui::floating::PopupAlign::Centre);
        assert_eq!(*placement.gap, 4.0);
    }
}
