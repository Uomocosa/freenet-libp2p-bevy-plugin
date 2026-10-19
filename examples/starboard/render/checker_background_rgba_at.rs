use crate::starboard;

#[must_use]
pub fn checker_background_rgba_at(x: usize, y: usize) -> [u8; 4] {
    let block_x = x.checked_div(starboard::render::CHECKER_CELLS).unwrap_or(0);
    let block_y = y.checked_div(starboard::render::CHECKER_CELLS).unwrap_or(0);
    let parity = block_x.saturating_add(block_y).checked_rem(2).unwrap_or(0);
    starboard::render::BOARD_CHECKER_RGBA
        .get(parity)
        .copied()
        .unwrap_or([0, 0, 0, 255])
}

#[cfg(test)]
mod tests {
    use super::checker_background_rgba_at;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let [dark, light] = starboard::render::BOARD_CHECKER_RGBA;
        assert_eq!(checker_background_rgba_at(0, 0), dark);
        assert_eq!(checker_background_rgba_at(7, 7), dark);
        assert_eq!(checker_background_rgba_at(8, 0), light);
        assert_eq!(checker_background_rgba_at(8, 8), dark);
    }
}
