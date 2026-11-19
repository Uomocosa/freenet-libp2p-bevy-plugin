use atomic_delegate_macros::atomic_delegates;
use bevy::math::{Rect, Vec2};
use bevy::ui::{ComputedNode, UiGlobalTransform};

use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupPlacement {
    pub anchor: Rect,
    pub popup: Vec2,
    pub area: Rect,
    pub preferred_side: ui::floating::PopupSide,
    pub cross_align: ui::floating::PopupAlign,
    pub gap: ui::floating::PopupGap,
}

#[atomic_delegates]
impl PopupPlacement {
    pub fn top_left(&self) -> Vec2 {}
    pub fn resolve_side(&self) -> ui::floating::PopupSide {}
    pub fn clamped_into(area: Rect, popup: Vec2, wanted: Vec2) -> Vec2 {}
    pub fn rect_of(node: &ComputedNode, transform: &UiGlobalTransform) -> Rect {}
}

impl PopupPlacement {
    pub(crate) fn horizontal_align_start(&self) -> f32 {
        match self.cross_align {
            ui::floating::PopupAlign::Start => self.anchor.min.x,
            ui::floating::PopupAlign::Centre => self.anchor.center().x - self.popup.x / 2.0,
            ui::floating::PopupAlign::End => self.anchor.max.x - self.popup.x,
        }
    }

    pub(crate) fn vertical_align_start(&self) -> f32 {
        match self.cross_align {
            ui::floating::PopupAlign::Start => self.anchor.min.y,
            ui::floating::PopupAlign::Centre => self.anchor.center().y - self.popup.y / 2.0,
            ui::floating::PopupAlign::End => self.anchor.max.y - self.popup.y,
        }
    }

    pub(crate) fn room_on(&self, side: ui::floating::PopupSide) -> f32 {
        match side {
            ui::floating::PopupSide::Below => self.area.max.y - self.anchor.max.y,
            ui::floating::PopupSide::Above => self.anchor.min.y - self.area.min.y,
            ui::floating::PopupSide::Right => self.area.max.x - self.anchor.max.x,
            ui::floating::PopupSide::Left => self.anchor.min.x - self.area.min.x,
        }
    }

    pub(crate) const fn extent_on(&self, side: ui::floating::PopupSide) -> f32 {
        match side {
            ui::floating::PopupSide::Below | ui::floating::PopupSide::Above => self.popup.y,
            ui::floating::PopupSide::Right | ui::floating::PopupSide::Left => self.popup.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::testing::{anchor, area, placement};
    use crate::ui;

    #[test]
    fn test_usage() {
        let placement = placement(
            area(Vec2::new(200.0, 200.0)),
            anchor(Vec2::new(90.0, 95.0), Vec2::new(110.0, 105.0)),
            Vec2::new(40.0, 18.0),
            ui::floating::PopupSide::Below,
            ui::floating::PopupAlign::Centre,
        );
        assert_eq!(placement.horizontal_align_start(), 80.0);
        assert_eq!(placement.vertical_align_start(), 91.0);
        assert_eq!(placement.room_on(ui::floating::PopupSide::Below), 95.0);
        assert_eq!(placement.extent_on(ui::floating::PopupSide::Right), 40.0);
    }
}
