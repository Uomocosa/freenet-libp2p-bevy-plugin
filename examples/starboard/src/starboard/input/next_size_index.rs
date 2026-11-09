use crate::starboard;

#[must_use]
pub fn next_size_index(size_index: u8, notches: i32) -> u8 {
    let last = u8::try_from(starboard::canvas::BRUSH_RADII.len().saturating_sub(1)).unwrap_or(0);
    let stepped = i32::from(size_index.min(last)).saturating_add(notches);
    u8::try_from(stepped.clamp(0, i32::from(last))).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::next_size_index;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let last = u8::try_from(starboard::canvas::BRUSH_RADII.len().saturating_sub(1))
            .expect("nine brush sizes fit u8");
        assert_eq!(next_size_index(5, 1), 6);
        assert_eq!(next_size_index(5, -1), 4);
        assert_eq!(next_size_index(0, -5), 0);
        assert_eq!(next_size_index(last, 3), last);
        assert_eq!(next_size_index(u8::MAX, 0), last);
    }
}
