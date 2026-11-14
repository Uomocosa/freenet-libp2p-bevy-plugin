use std::str::FromStr;

use bevy::math::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowOrigin(pub IVec2);

impl FromStr for WindowOrigin {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
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
        Ok(Self(IVec2::new(x, y)))
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::IVec2;

    use super::WindowOrigin;

    #[test]
    fn test_usage() {
        assert_eq!(
            "481, 0".parse::<WindowOrigin>(),
            Ok(WindowOrigin(IVec2::new(481, 0)))
        );
        assert!("481".parse::<WindowOrigin>().is_err());
    }
}
