use bevy::prelude::{MessageWriter, Resource};

use crate::starboard;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatestPaintedCell {
    pub cell: Option<starboard::canvas::SubCellPos>,
    pub earlier_points: [starboard::canvas::SubCellPos; 2],
    pub action: Option<starboard::canvas::DrawAction>,
    pub stroke_rank: u32,
}

impl LatestPaintedCell {
    pub(crate) fn flush_stroke(
        &mut self,
        brush: starboard::input::OwnBrush,
        draws: &mut MessageWriter<starboard::input::PendingLocalDraw>,
    ) {
        let (action, stroke_rank) = (self.action, self.stroke_rank);
        let lists = starboard::input::finish_stroke(self);
        if let Some(action) = action {
            starboard::input::PendingLocalDraw::write_lists(
                draws,
                brush,
                action,
                stroke_rank,
                lists,
            );
        }
    }
}

// no test_usage necessary
