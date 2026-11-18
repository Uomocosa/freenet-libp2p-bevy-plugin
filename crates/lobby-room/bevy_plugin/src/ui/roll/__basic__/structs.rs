use bevy::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RollNumberStyle {
    pub font_size: f32,
    pub digit_width: f32,
    pub row_height: f32,
    pub digit_count: u8,
    pub text_color: Color,
}

// no test_usage necessary
