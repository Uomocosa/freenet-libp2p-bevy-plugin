use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorUpdate {
    pub cell: starboard::canvas::CellPos,
    pub size_index: u8,
}
