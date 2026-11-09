use bevy::prelude::{Resource, Vec2};

use crate::starboard;

#[derive(Resource, Debug, Clone, Copy)]
pub struct CanvasView {
    pub scale: f32,
    pub offset: Vec2,
}

impl Default for CanvasView {
    fn default() -> Self {
        Self::fit(Vec2::splat(1024.0))
    }
}

impl CanvasView {
    #[must_use]
    pub fn fit(window: Vec2) -> Self {
        let side = f32::from(u16::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(1024));
        let scale = window.x.min(window.y).max(1.0) / side;
        let board = side * scale;
        Self {
            scale,
            offset: Vec2::new((window.x - board) * 0.5, (window.y - board) * 0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::CanvasView;

    #[test]
    fn test_usage() {
        let wide = CanvasView::fit(Vec2::new(1600.0, 900.0));
        let side = 1024.0_f32;
        assert!((wide.scale - 900.0 / side).abs() < 0.000_1);
        assert!((wide.offset.x - 350.0).abs() < 0.000_1);
        assert!(wide.offset.y.abs() < 0.000_1);

        let tall = CanvasView::fit(Vec2::new(900.0, 1600.0));
        assert!(tall.offset.x.abs() < 0.000_1);
        assert!((tall.offset.y - 350.0).abs() < 0.000_1);

        let tiny = CanvasView::fit(Vec2::ZERO);
        assert!(tiny.scale > 0.0);
    }
}
