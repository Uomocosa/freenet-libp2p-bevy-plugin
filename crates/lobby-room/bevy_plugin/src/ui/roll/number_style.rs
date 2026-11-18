use bevy::prelude::{Color, FontSize, TextFont};
use bevy::text::LineHeight;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberStyle {
    pub font_size: f32,
    pub row_height: f32,
    pub text_color: Color,
}

#[rustfmt::skip]
impl NumberStyle {
    #[must_use] pub fn text_font(&self) -> TextFont { TextFont::from_font_size(FontSize::Px(self.font_size)) }
    #[must_use] pub const fn line_height(&self) -> LineHeight { LineHeight::Px(self.row_height) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Color, FontSize, TextFont};
    use bevy::text::LineHeight;

    use super::NumberStyle;

    #[test]
    fn test_usage() {
        let style = NumberStyle {
            font_size: 11.0,
            row_height: 13.0,
            text_color: Color::WHITE,
        };
        assert_eq!(
            style.text_font(),
            TextFont::from_font_size(FontSize::Px(11.0))
        );
        assert_eq!(style.line_height(), LineHeight::Px(13.0));
    }
}
