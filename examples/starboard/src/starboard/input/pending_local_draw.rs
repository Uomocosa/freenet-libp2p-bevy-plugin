use bevy::prelude::{Message, MessageWriter};
use derive_more::Deref;

use crate::starboard;

#[derive(Message, Debug, Clone, Deref)]
pub struct PendingLocalDraw(pub starboard::canvas::DrawOp);

impl PendingLocalDraw {
    pub(crate) fn write_lists(
        draws: &mut MessageWriter<Self>,
        brush: starboard::input::OwnBrush,
        action: starboard::canvas::DrawAction,
        stroke_rank: u32,
        lists: Vec<Vec<starboard::canvas::SubCellPos>>,
    ) {
        for points in lists {
            draws.write(Self(starboard::input::draw_op(
                brush,
                action,
                points,
                stroke_rank,
            )));
        }
    }
}

// no test_usage necessary
