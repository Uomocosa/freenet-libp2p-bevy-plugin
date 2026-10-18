use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};

use crate::starboard;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct LatestPaintedCell(pub Option<starboard::canvas::CellPos>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default, Deref, DerefMut)]
pub struct CursorCell(pub Option<starboard::canvas::CellPos>);
