use crate::starboard;

/// Parses `from-to` cells, e.g. `10.20-30.102`, into a stroke on the board.
///
/// # Errors
///
/// Returns a message naming the offending part when the text is not two `x.y`
/// cells, or when a cell is not a number or lies outside the canvas.
pub fn parse_stroke(raw: &str) -> Result<starboard::canvas::Stroke, String> {
    let Some((from, to)) = raw.split_once('-') else {
        return Err(format!("expected a stroke like 10.20-30.102, got '{raw}'"));
    };
    if to.contains('-') {
        return Err(format!("expected a stroke like 10.20-30.102, got '{raw}'"));
    }
    Ok(starboard::canvas::Stroke {
        from: parse_cell(from)?,
        to: parse_cell(to)?,
    })
}

// needed helper: one `x.y` cell, inside the canvas
fn parse_cell(raw: &str) -> Result<starboard::canvas::CellPos, String> {
    let Some((x, y)) = raw.split_once('.') else {
        return Err(format!("expected a cell like 10.20, got '{raw}'"));
    };
    if y.contains('.') {
        return Err(format!("expected a cell like 10.20, got '{raw}'"));
    }
    let (Ok(x), Ok(y)) = (x.parse::<u32>(), y.parse::<u32>()) else {
        return Err(format!("cell '{raw}' is not a number"));
    };
    let side = u32::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(u32::MAX);
    if x >= side || y >= side {
        return Err(format!(
            "cell '{raw}' is outside the canvas (0..{})",
            starboard::canvas::CANVAS_SIDE
        ));
    }
    Ok(starboard::canvas::CellPos {
        x: u16::try_from(x).unwrap_or(0),
        y: u16::try_from(y).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_stroke;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let stroke = parse_stroke("10.20-30.102").expect("stroke");
        assert_eq!(stroke.from, starboard::canvas::CellPos { x: 10, y: 20 });
        assert_eq!(stroke.to, starboard::canvas::CellPos { x: 30, y: 102 });
    }

    #[test]
    fn a_stroke_without_two_cells_is_rejected() {
        assert!(parse_stroke("10.20").is_err());
        assert!(parse_stroke("10.20-30.102-40.5").is_err());
    }

    #[test]
    fn a_cell_without_two_coordinates_is_rejected() {
        assert!(parse_stroke("10-30.102").is_err());
        assert!(parse_stroke("10.20.5-30.102").is_err());
    }

    #[test]
    fn a_cell_that_is_not_a_number_is_rejected() {
        assert!(parse_stroke("a.b-30.102").is_err());
        assert!(parse_stroke("10.20-30.x").is_err());
    }

    #[test]
    fn a_cell_outside_the_canvas_is_rejected() {
        let error = parse_stroke("10.20-9999.5").expect_err("outside");
        assert!(error.contains("9999.5"));
        assert!(error.contains("outside the canvas"));
    }
}
