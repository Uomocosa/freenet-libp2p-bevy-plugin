use crate::starboard;

pub const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

pub const SHAPES: [starboard::identity::Shape; 6] = [
    starboard::identity::Shape::Circle,
    starboard::identity::Shape::Star,
    starboard::identity::Shape::Square,
    starboard::identity::Shape::Triangle,
    starboard::identity::Shape::Diamond,
    starboard::identity::Shape::Hexagon,
];

pub const PALETTE: [[u8; 3]; 12] = [
    [230, 57, 70],
    [244, 162, 97],
    [233, 196, 106],
    [138, 201, 38],
    [42, 157, 143],
    [0, 180, 216],
    [58, 134, 255],
    [131, 56, 236],
    [255, 0, 110],
    [251, 86, 7],
    [6, 214, 160],
    [255, 209, 102],
];

pub const COMBINATION_COUNT: usize = SHAPES.len() * PALETTE.len();
