use bevy::prelude::*;

use crate::ui;

#[must_use]
pub fn roll_window_width(app: &mut App) -> Option<f32> {
    let mut numbers = app.world_mut().query::<(&ui::roll::RollNumber, &Node)>();
    let (_, node) = numbers.single(app.world()).ok()?;
    let Val::Px(width) = node.width else {
        return None;
    };
    Some(width)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::testing;
    use crate::ui;

    use super::roll_window_width;

    #[test]
    fn test_usage() {
        let style = ui::roll::NumberStyle {
            font_size: 11.0,
            row_height: 13.0,
            text_color: Color::WHITE,
        };
        let mut app = testing::roll_app_with_layout(style);
        assert!(roll_window_width(&mut app).is_some());
    }
}
