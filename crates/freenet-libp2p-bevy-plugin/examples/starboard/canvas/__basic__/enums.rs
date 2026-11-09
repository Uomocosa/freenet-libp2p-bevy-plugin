use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DrawAction {
    Paint(starboard::identity::PaletteColor),
    Erase,
}
