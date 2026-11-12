use bevy::prelude::{MouseButton, Vec2};

use crate::starboard;

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPress {
    pub button: MouseButton,
    pub started_at_secs: f32,
    pub press_screen: Vec2,
    pub path: Vec<starboard::canvas::SubCellPos>,
    pub stroke_rank: u32,
}

impl PendingPress {
    #[must_use]
    pub fn begin(
        button: MouseButton,
        cell: starboard::canvas::SubCellPos,
        screen: Vec2,
        now_secs: f32,
        stroke_rank: u32,
    ) -> Self {
        Self {
            button,
            started_at_secs: now_secs,
            press_screen: screen,
            path: vec![cell],
            stroke_rank,
        }
    }

    pub fn record(&mut self, cell: starboard::canvas::SubCellPos) {
        if self.path.last() != Some(&cell) {
            self.path.push(cell);
        }
    }

    #[must_use]
    pub fn has_become_stroke(&self, screen: Vec2, now_secs: f32) -> bool {
        screen.distance(self.press_screen) > starboard::input::DRAG_SLOP_PX
            || now_secs - self.started_at_secs >= starboard::input::DOT_MAX_HOLD_SECS
    }

    #[must_use]
    pub fn segments(&self) -> Vec<starboard::canvas::Stroke> {
        self.path
            .iter()
            .zip(self.path.iter().skip(1))
            .map(|(from, to)| starboard::canvas::Stroke {
                from: *from,
                to: *to,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{MouseButton, Vec2};

    use super::PendingPress;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let cell = starboard::canvas::SubCellPos { x: 80, y: 80 };
        let mut press = PendingPress::begin(MouseButton::Left, cell, Vec2::splat(10.0), 0.0, 7);
        press.record(cell);
        assert!(!press.has_become_stroke(Vec2::splat(10.0), 0.1));
        assert!(press.has_become_stroke(Vec2::splat(10.0), 0.3));
        press.record(starboard::canvas::SubCellPos { x: 160, y: 80 });
        assert_eq!(press.segments().len(), 1);
    }
}
