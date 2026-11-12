use bevy::prelude::{UVec2, Vec2};

use crate::starboard;

#[must_use]
pub fn sub_cell_under_cursor(
    cursor: Vec2,
    view: &starboard::render::CanvasView,
) -> Option<starboard::canvas::SubCellPos> {
    if view.scale <= 0.0 {
        return None;
    }
    let local_x = (cursor.x - view.offset.x) / view.scale;
    let local_y = (cursor.y - view.offset.y) / view.scale;
    let side = f32::from(u16::try_from(starboard::canvas::CANVAS_SIDE).ok()?);
    if local_x < 0.0 || local_y < 0.0 || local_x >= side || local_y >= side {
        return None;
    }
    let steps = f32::from(u8::try_from(starboard::canvas::SUBCELL_STEPS).ok()?);
    let last = u32::try_from(starboard::canvas::CANVAS_SIDE)
        .ok()?
        .saturating_mul(starboard::canvas::SUBCELL_STEPS)
        .saturating_sub(1);
    let scaled = Vec2::new(local_x * steps, local_y * steps);
    let point = scaled.round().as_uvec2().min(UVec2::splat(last));
    Some(starboard::canvas::SubCellPos {
        x: point.x,
        y: point.y,
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::sub_cell_under_cursor;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let view = starboard::render::CanvasView::fit(Vec2::splat(1024.0));
        assert_eq!(
            sub_cell_under_cursor(Vec2::new(0.0, 0.0), &view),
            Some(starboard::canvas::SubCellPos { x: 0, y: 0 })
        );
        assert_eq!(
            sub_cell_under_cursor(Vec2::new(10.26, 10.2), &view),
            Some(starboard::canvas::SubCellPos { x: 82, y: 82 })
        );
        assert_eq!(
            sub_cell_under_cursor(Vec2::new(1023.97, 1023.97), &view),
            Some(starboard::canvas::SubCellPos { x: 8191, y: 8191 })
        );
        assert_eq!(sub_cell_under_cursor(Vec2::new(1024.0, 10.0), &view), None);
        assert_eq!(sub_cell_under_cursor(Vec2::new(-1.0, 10.0), &view), None);
    }
}
