use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Shape {
    Circle,
    Star,
    Square,
    Triangle,
    Diamond,
    Hexagon,
}
