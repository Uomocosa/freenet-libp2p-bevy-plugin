use crate::starboard;

#[must_use]
pub const fn draw_op(
    brush: starboard::input::OwnBrush,
    action: starboard::canvas::DrawAction,
    stroke: starboard::canvas::Stroke,
    stroke_rank: u32,
) -> starboard::canvas::DrawOp {
    starboard::canvas::DrawOp {
        action,
        shape: brush.shape,
        size_index: brush.size_index,
        from: stroke.from,
        to: stroke.to,
        stroke_rank,
    }
}

#[cfg(test)]
mod tests {
    use super::draw_op;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let brush = starboard::input::OwnBrush {
            shape: starboard::identity::Shape::Star,
            color: starboard::identity::PaletteColor(3),
            size_index: 2,
        };
        let cell = starboard::canvas::SubCellPos { x: 80, y: 88 };
        let stroke = starboard::canvas::Stroke {
            from: cell,
            to: cell,
        };
        let paint = draw_op(
            brush,
            starboard::canvas::DrawAction::Paint(brush.color),
            stroke,
            3,
        );
        assert_eq!(paint.stroke_rank, 3);
        assert_eq!(paint.shape, starboard::identity::Shape::Star);
        assert_eq!(paint.size_index, 2);
        assert_eq!(paint.from, cell);
        assert_eq!(paint.to, cell);

        let erase = draw_op(brush, starboard::canvas::DrawAction::Erase, stroke, 4);
        assert_eq!(erase.action, starboard::canvas::DrawAction::Erase);
    }
}
