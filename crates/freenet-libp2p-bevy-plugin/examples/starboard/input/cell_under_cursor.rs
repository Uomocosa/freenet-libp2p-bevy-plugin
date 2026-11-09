use bevy::prelude::Vec2;

use crate::starboard;

#[must_use]
pub fn cell_under_cursor(
    cursor: Vec2,
    view: &starboard::render::CanvasView,
) -> Option<starboard::canvas::CellPos> {
    if view.scale <= 0.0 {
        return None;
    }
    let local_x = (cursor.x - view.offset.x) / view.scale;
    let local_y = (cursor.y - view.offset.y) / view.scale;
    if local_x < 0.0 || local_y < 0.0 {
        return None;
    }
    let cell = Vec2::new(local_x, local_y).as_uvec2();
    let side = u32::try_from(starboard::canvas::CANVAS_SIDE).ok()?;
    if cell.x >= side || cell.y >= side {
        return None;
    }
    Some(starboard::canvas::CellPos {
        x: u16::try_from(cell.x).ok()?,
        y: u16::try_from(cell.y).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::cell_under_cursor;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let view = starboard::render::CanvasView::fit(Vec2::splat(1024.0));
        assert_eq!(
            cell_under_cursor(Vec2::new(0.0, 0.0), &view),
            Some(starboard::canvas::CellPos { x: 0, y: 0 })
        );
        assert_eq!(
            cell_under_cursor(Vec2::new(1023.9, 1023.9), &view),
            Some(starboard::canvas::CellPos { x: 1023, y: 1023 })
        );
        assert_eq!(cell_under_cursor(Vec2::new(1024.0, 10.0), &view), None);
        assert_eq!(cell_under_cursor(Vec2::new(-1.0, 10.0), &view), None);
    }

    #[test]
    fn the_letterbox_margin_is_outside_the_canvas() {
        let view = starboard::render::CanvasView::fit(Vec2::new(1600.0, 900.0));
        assert_eq!(cell_under_cursor(Vec2::new(100.0, 450.0), &view), None);

        let inside = view.offset + Vec2::splat(view.scale * 0.5);
        assert_eq!(
            cell_under_cursor(inside, &view),
            Some(starboard::canvas::CellPos { x: 0, y: 0 })
        );
    }
}
