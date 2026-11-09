use bevy::prelude::{Vec2, Vec3};

use crate::starboard;

#[must_use]
pub fn cell_center(
    cell: starboard::canvas::CellPos,
    view: &starboard::render::CanvasView,
    window: Vec2,
) -> Vec3 {
    let x = (f32::from(cell.x) + 0.5).mul_add(view.scale, view.offset.x);
    let y = (f32::from(cell.y) + 0.5).mul_add(view.scale, view.offset.y);
    Vec3::new(
        (-0.5_f32).mul_add(window.x, x),
        0.5_f32.mul_add(window.y, -y),
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::cell_center;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let view = starboard::render::CanvasView::fit(Vec2::splat(1024.0));
        let center = cell_center(
            starboard::canvas::CellPos { x: 300, y: 200 },
            &view,
            Vec2::splat(1024.0),
        );
        assert!((center.x - -211.5).abs() < 0.000_1);
        assert!((center.y - 311.5).abs() < 0.000_1);
        assert!((center.z - 1.0).abs() < 0.000_1);
    }
}
