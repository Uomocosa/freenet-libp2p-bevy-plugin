use crate::testing;

pub fn from_str(raw: &str) -> Result<testing::harness::WindowOrigin, String> {
    let (x, y) = raw
        .split_once(',')
        .ok_or_else(|| format!("expected x,y, got {raw}"))?;
    let x = x
        .trim()
        .parse::<i32>()
        .map_err(|error| format!("x: {error}"))?;
    let y = y
        .trim()
        .parse::<i32>()
        .map_err(|error| format!("y: {error}"))?;
    Ok(testing::harness::WindowOrigin(bevy::math::IVec2::new(x, y)))
}

// no test_usage necessary
