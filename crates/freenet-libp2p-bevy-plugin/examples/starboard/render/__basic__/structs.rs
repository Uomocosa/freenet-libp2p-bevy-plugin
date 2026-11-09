use crate::starboard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushImage {
    pub side: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrushOutline {
    pub image: starboard::render::BrushImage,
    pub screen_side: f32,
}
