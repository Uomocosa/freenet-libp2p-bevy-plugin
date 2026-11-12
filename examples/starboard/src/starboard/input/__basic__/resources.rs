use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::starboard;

/// The active stroke's lookahead window. `cell` is the newest recorded point (`None` when no
/// stroke is active), `earlier_points` are the two before it, and `action` is kept because a
/// release arrives with no button held.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatestPaintedCell {
    pub cell: Option<starboard::canvas::SubCellPos>,
    pub earlier_points: [starboard::canvas::SubCellPos; 2],
    pub action: Option<starboard::canvas::DrawAction>,
    pub stroke_rank: u32,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct CursorCell(pub Option<starboard::canvas::CellPos>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct CursorPoint(pub Option<starboard::canvas::SubCellPos>);

#[derive(Resource, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct PendingPressState(pub Option<starboard::input::PendingPress>);
