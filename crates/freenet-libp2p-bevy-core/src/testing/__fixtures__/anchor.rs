use bevy::math::{Rect, Vec2};

#[must_use]
pub const fn anchor(min: Vec2, max: Vec2) -> Rect {
    Rect::from_corners(min, max)
}

#[cfg(test)]
mod tests {
    use bevy::math::{Rect, Vec2};

    use super::anchor;

    #[test]
    fn test_usage() {
        assert_eq!(
            anchor(Vec2::new(90.0, 95.0), Vec2::new(110.0, 105.0)),
            Rect::from_corners(Vec2::new(90.0, 95.0), Vec2::new(110.0, 105.0))
        );
    }
}
