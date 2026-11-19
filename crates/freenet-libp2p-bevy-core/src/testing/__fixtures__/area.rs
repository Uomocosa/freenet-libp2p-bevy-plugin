use bevy::math::{Rect, Vec2};

#[must_use]
pub const fn area(size: Vec2) -> Rect {
    Rect::from_corners(Vec2::ZERO, size)
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};

    use super::area;

    #[test]
    fn test_usage() {
        assert_eq!(
            area(Vec2::new(960.0, 540.0)),
            Rect::from_corners(Vec2::ZERO, Vec2::new(960.0, 540.0))
        );
    }
}
