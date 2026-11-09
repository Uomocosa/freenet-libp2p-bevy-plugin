use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::starboard;

pub fn scroll_brush_size(
    scroll: Res<AccumulatedMouseScroll>,
    mut brush: ResMut<starboard::input::OwnBrush>,
) {
    let scroll = scroll.into_inner();
    let notches = match scroll.delta.y.partial_cmp(&0.0) {
        Some(std::cmp::Ordering::Greater) => 1,
        Some(std::cmp::Ordering::Less) => -1,
        _ => 0,
    };
    if notches == 0 {
        return;
    }
    let next = starboard::input::next_size_index(brush.size_index, notches);
    if next != brush.size_index {
        brush.size_index = next;
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
    use bevy::prelude::*;

    use super::scroll_brush_size;
    use crate::starboard;

    fn app_with_scroll(delta_y: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(starboard::input::OwnBrush::default());
        app.insert_resource(AccumulatedMouseScroll {
            unit: MouseScrollUnit::Line,
            delta: Vec2::new(0.0, delta_y),
        });
        app.add_systems(Update, scroll_brush_size);
        app.update();
        app
    }

    #[test]
    fn test_usage() {
        let start = starboard::canvas::DEFAULT_SIZE_INDEX;
        let up = app_with_scroll(1.0);
        let brush = up.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(brush.size_index, start.saturating_add(1));

        let down = app_with_scroll(-1.0);
        let brush = down.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(brush.size_index, start.saturating_sub(1));

        let still = app_with_scroll(0.0);
        let brush = still.world().resource::<starboard::input::OwnBrush>();
        assert_eq!(brush.size_index, start);
    }
}
