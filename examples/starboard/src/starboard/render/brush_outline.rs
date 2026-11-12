use crate::starboard;

#[derive(Debug, Clone, PartialEq)]
pub struct BrushOutline {
    pub image: starboard::render::BrushImage,
    pub screen_side: f32,
}

impl BrushOutline {
    pub(crate) fn ceil_to_u16(length: f32) -> u16 {
        (0..u16::MAX)
            .find(|candidate| f32::from(*candidate) >= length)
            .unwrap_or(u16::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::BrushOutline;

    #[test]
    fn test_usage() {
        assert_eq!(BrushOutline::ceil_to_u16(2.5), 3);
        assert_eq!(BrushOutline::ceil_to_u16(4.0), 4);
    }
}
