use bevy::prelude::*;

use crate::starboard;

#[must_use]
pub fn cursor_label(color: Color, text: &str) -> impl Bundle {
    (
        starboard::render::CursorLabel,
        Text2d::new(text),
        TextFont::from_font_size(starboard::render::LABEL_FONT_SIZE),
        TextColor(color),
        TextBackgroundColor(starboard::render::LABEL_BACKGROUND),
        Transform::default(),
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::cursor_label;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let label = world.spawn(cursor_label(Color::WHITE, "alice")).id();
        assert!(world.get::<starboard::render::CursorLabel>(label).is_some());
        assert_eq!(
            world.get::<TextColor>(label),
            Some(&TextColor(Color::WHITE))
        );
        assert_eq!(
            world.get::<Text2d>(label).map(|text| text.as_str()),
            Some("alice")
        );
    }
}
